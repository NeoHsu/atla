use anyhow::Context;
use atla_core::{ConfluenceSpaceCreate, ConfluenceSpaceSearch, ConfluenceSpaceUpdate};

use crate::cli::{GlobalArgs, OutputFormat, SpaceAction, SpaceCommand};
use crate::context::AppContext;

use super::format::{
    print_deleted, print_space, print_spaces, print_spaces_with_footer, read_body,
};

pub(super) async fn run_space(command: SpaceCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        SpaceAction::List {
            key,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let query_hash = crate::pagination::query_hash(
                "confluence.space.list",
                &[("key", key.clone().unwrap_or_default())],
            );
            let cursor = crate::pagination::decode_confluence_cursor_token(
                page_token.as_deref(),
                "confluence.space.list",
                query_hash.clone(),
            )?;
            let search = ConfluenceSpaceSearch {
                key,
                limit: if all { u32::MAX } else { limit.clamp(1, 250) },
                cursor,
            };

            if global.dry_run {
                let mut url = format!(
                    "{}/wiki/api/v2/spaces?limit={}",
                    profile.instance.trim_end_matches('/'),
                    search.limit
                );
                if let Some(key) = &search.key {
                    url.push_str(&format!("&keys={key}"));
                }
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let page = client.list_spaces(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence spaces from {}",
                    client.instance_url()
                )
            })?;

            let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
                crate::pagination::confluence_cursor_next_token(
                    "confluence.space.list",
                    page.next_cursor.clone(),
                    query_hash,
                )?
            } else {
                None
            };
            let next_command = next_cli_token.as_ref().map(|token| {
                let mut parts = vec![
                    "atla".to_owned(),
                    "confluence".to_owned(),
                    "space".to_owned(),
                    "list".to_owned(),
                ];
                if let Some(key) = &search.key {
                    parts.push("--key".to_owned());
                    parts.push(crate::pagination::quote(key));
                }
                crate::pagination::next_command(parts, limit, token)
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(&serde_json::json!({
                    "results": page.results,
                    "pagination": { "isLast": page.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command }
                }))?,
                OutputFormat::Table => print_spaces_with_footer(
                    &page.results,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_spaces(&page.results, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        SpaceAction::View { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/spaces?keys={}&limit=1",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let space = client.get_space(&key).await.with_context(|| {
                format!(
                    "failed to load Confluence space `{key}` from {}",
                    client.instance_url()
                )
            })?;
            let space =
                space.ok_or_else(|| anyhow::anyhow!("Confluence space `{key}` was not found"))?;

            print_space(&space, global)?;
        }
        SpaceAction::Create {
            name,
            key,
            alias,
            description,
            description_file,
            private,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            if key.is_none() && alias.is_none() {
                anyhow::bail!("provide --key or --alias");
            }
            let description = read_body(description, description_file.as_deref())?;
            let create = ConfluenceSpaceCreate {
                key,
                alias,
                name,
                description,
                private,
            };

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/api/v2/spaces using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let space = client.create_space(&create).await.with_context(|| {
                format!(
                    "failed to create Confluence space in {}",
                    client.instance_url()
                )
            })?;

            print_space(&space, global)?;
        }
        SpaceAction::Update {
            key,
            name,
            description,
            description_file,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let description = read_body(description, description_file.as_deref())?;
            if name.is_none() && description.is_none() {
                anyhow::bail!("provide --name, --description, or --description-file");
            }
            let update = ConfluenceSpaceUpdate {
                key,
                name,
                description,
            };

            if global.dry_run {
                println!(
                    "Would PUT {}/wiki/rest/api/space/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    update.key
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let space = client.update_space(&update).await.with_context(|| {
                format!(
                    "failed to update Confluence space `{}` from {}",
                    update.key,
                    client.instance_url()
                )
            })?;

            print_space(&space, global)?;
        }
        SpaceAction::Delete { key, yes } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/rest/api/space/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                return Ok(());
            }
            if !yes {
                anyhow::bail!("refusing to delete space `{key}` without --yes");
            }

            let client = ctx.confluence_client()?;
            client.delete_space(&key).await.with_context(|| {
                format!(
                    "failed to delete Confluence space `{key}` from {}",
                    client.instance_url()
                )
            })?;
            print_deleted("space", &key, global)?;
        }
    }

    Ok(())
}
