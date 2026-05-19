use anyhow::Context;
use atla_core::{ConfluenceSpaceCreate, ConfluenceSpaceSearch, ConfluenceSpaceUpdate};

use crate::cli::{GlobalArgs, SpaceAction, SpaceCommand};
use crate::context::AppContext;

use super::format::{print_deleted, print_space, print_spaces, read_body};

pub(super) async fn run_space(command: SpaceCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        SpaceAction::List { key, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceSpaceSearch {
                key,
                limit: limit.clamp(1, 250),
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

            print_spaces(&page.results, global)?;
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
            let space = client.get_space_by_key(&key).await.with_context(|| {
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
