use anyhow::Context;
use atla_core::{
    ConfluenceAttachmentSearch, ConfluenceContentTreeSearch, ConfluencePageCopy,
    ConfluencePageCreate, ConfluencePageSearch, ConfluencePageUpdate,
};

use crate::cli::{ContentViewFormat, GlobalArgs, OutputFormat, PageAction, PageCommand};
use crate::context::AppContext;

use super::format::{
    open_web_url, prepare_optional_body, prepare_required_body, print_attachments,
    print_content_nodes, print_content_nodes_with_footer, print_page, print_page_body,
    print_page_body_markdown, print_page_with_attachments, print_pages, print_pages_with_footer,
    read_body, resolve_required_space_id, resolve_space_id, status_from_draft,
    view_format_body_representation,
};
use super::page_comment::run_page_comment;
use super::page_label::run_page_label;

pub(super) async fn run_page(command: PageCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        PageAction::Create {
            space,
            space_id,
            title,
            parent,
            body,
            body_file,
            representation,
            draft,
            private,
            root_level,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                if let Some(space) = &space {
                    let space_url = format!(
                        "{}/wiki/api/v2/spaces?keys={space}&limit=1",
                        profile.instance.trim_end_matches('/')
                    );
                    println!("Would GET {space_url} using profile `{profile_name}`");
                }
                println!(
                    "Would POST {}/wiki/api/v2/pages using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let (body, representation) =
                prepare_optional_body(read_body(body, body_file.as_deref())?, representation)?;
            let client = ctx.confluence_client()?;
            let space_id = resolve_required_space_id(&client, space.as_deref(), space_id).await?;
            let page = client
                .create_page(&ConfluencePageCreate {
                    space_id,
                    title,
                    parent_id: parent,
                    body,
                    representation,
                    status: status_from_draft(draft),
                    private: private.then_some(true),
                    root_level: root_level.then_some(true),
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to create Confluence page in {}",
                        client.instance_url()
                    )
                })?;

            print_page(&page, global)?;
        }
        PageAction::List {
            space,
            space_id,
            title,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let limit = if all { u32::MAX } else { limit.clamp(1, 250) };

            if global.dry_run {
                if let Some(space) = &space {
                    let space_url = format!(
                        "{}/wiki/api/v2/spaces?keys={space}&limit=1",
                        profile.instance.trim_end_matches('/')
                    );
                    println!("Would GET {space_url} using profile `{profile_name}`");
                }

                let mut url = format!(
                    "{}/wiki/api/v2/pages?limit={limit}",
                    profile.instance.trim_end_matches('/')
                );
                if let Some(space_id) = &space_id {
                    url.push_str(&format!("&space-id={space_id}"));
                } else if space.is_some() {
                    url.push_str("&space-id=<resolved-space-id>");
                }
                if let Some(title) = &title {
                    url.push_str(&format!("&title={title}"));
                }
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let resolved_space_id = resolve_space_id(&client, space.as_deref(), space_id).await?;
            let cursor = crate::pagination::decode_confluence_page_token(
                page_token.as_deref(),
                resolved_space_id.as_deref(),
                title.as_deref(),
            )?;
            let search = ConfluencePageSearch {
                space_id: resolved_space_id,
                title,
                limit,
                cursor,
            };
            let page = client.list_pages(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence pages from {}",
                    client.instance_url()
                )
            })?;

            let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
                crate::pagination::confluence_page_next_token(
                    page.next_cursor.clone(),
                    search.space_id.as_deref(),
                    search.title.as_deref(),
                )?
            } else {
                None
            };
            let next_command = next_cli_token.as_ref().map(|token| {
                crate::pagination::confluence_page_next_command(
                    search.space_id.as_deref(),
                    search.title.as_deref(),
                    limit,
                    token,
                )
            });

            match global.output.unwrap_or(crate::cli::OutputFormat::Table) {
                crate::cli::OutputFormat::Json => crate::output::print_json(&serde_json::json!({
                    "results": page.results,
                    "pagination": {
                        "isLast": page.is_last.unwrap_or(true),
                        "nextPageToken": next_cli_token,
                        "nextCommand": next_command,
                    }
                }))?,
                crate::cli::OutputFormat::Table => {
                    let footer = next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer);
                    print_pages_with_footer(&page.results, global, footer)?;
                }
                crate::cli::OutputFormat::Csv | crate::cli::OutputFormat::Keys => {
                    print_pages(&page.results, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        PageAction::View {
            id,
            web,
            format,
            with_attachments,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/pages/{}",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                println!("Would GET {url} using profile `{profile_name}`");
                if with_attachments {
                    let att_url = format!(
                        "{}/wiki/api/v2/pages/{}/attachments",
                        profile.instance.trim_end_matches('/'),
                        id
                    );
                    println!("Would GET {att_url} using profile `{profile_name}`");
                }
                return Ok(());
            }

            if web {
                open_web_url(&format!(
                    "{}/wiki/pages/viewpage.action?pageId={}",
                    profile.instance.trim_end_matches('/'),
                    id
                ))?;
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let page = client
                .get_page_with_body_format(&id, format.and_then(view_format_body_representation))
                .await
                .with_context(|| {
                    format!(
                        "failed to load Confluence page `{id}` from {}",
                        client.instance_url()
                    )
                })?;

            if with_attachments {
                let search = ConfluenceAttachmentSearch {
                    page_id: id.clone(),
                    filename: None,
                    limit: 250,
                    cursor: None,
                };
                let attachments = client
                    .list_page_attachments(&search)
                    .await
                    .with_context(|| format!("failed to list attachments for page `{id}`"))?;
                if matches!(format, Some(ContentViewFormat::Markdown)) {
                    print_page_body_markdown(&page)?;
                } else if format.is_some() {
                    print_page_body(&page)?;
                } else {
                    print_page_with_attachments(&page, &attachments.results, global)?;
                    return Ok(());
                }
                // body-format view: print attachments separately (body is already printed above)
                if attachments.results.is_empty() {
                    eprintln!("(no attachments)");
                } else {
                    print_attachments(&attachments.results, global)?;
                }
            } else if matches!(format, Some(ContentViewFormat::Markdown)) {
                print_page_body_markdown(&page)?;
            } else if format.is_some() {
                print_page_body(&page)?;
            } else {
                print_page(&page, global)?;
            }
        }
        PageAction::Children {
            id,
            depth,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let depth = depth.map(|depth| depth.clamp(1, 100));
            let query_hash = crate::pagination::query_hash(
                "confluence.page.children",
                &[
                    ("pageId", id.clone()),
                    ("depth", depth.map(|d| d.to_string()).unwrap_or_default()),
                ],
            );
            let cursor = crate::pagination::decode_confluence_cursor_token(
                page_token.as_deref(),
                "confluence.page.children",
                query_hash.clone(),
            )?;
            let search = ConfluenceContentTreeSearch {
                page_id: id,
                limit: if all { u32::MAX } else { limit.clamp(1, 250) },
                depth,
                cursor,
            };

            if global.dry_run {
                let endpoint = if let Some(depth) = search.depth {
                    format!(
                        "{}/wiki/api/v2/pages/{}/descendants?limit={}&depth={depth}",
                        profile.instance.trim_end_matches('/'),
                        search.page_id,
                        search.limit
                    )
                } else {
                    format!(
                        "{}/wiki/api/v2/pages/{}/direct-children?limit={}",
                        profile.instance.trim_end_matches('/'),
                        search.page_id,
                        search.limit
                    )
                };
                println!("Would GET {endpoint} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let children = client.list_page_children(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence page children for `{}` from {}",
                    search.page_id,
                    client.instance_url()
                )
            })?;

            let next_cli_token = if !all && matches!(children.is_last, Some(false)) {
                crate::pagination::confluence_cursor_next_token(
                    "confluence.page.children",
                    children.next_cursor.clone(),
                    query_hash,
                )?
            } else {
                None
            };
            let next_command = next_cli_token.as_ref().map(|token| {
                let mut parts = vec![
                    "atla".to_owned(),
                    "confluence".to_owned(),
                    "page".to_owned(),
                    "children".to_owned(),
                    crate::pagination::quote(&search.page_id),
                ];
                if let Some(depth) = search.depth {
                    parts.push("--depth".to_owned());
                    parts.push(depth.to_string());
                }
                crate::pagination::next_command(parts, limit, token)
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(&serde_json::json!({
                    "results": children.results,
                    "pagination": {"isLast": children.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}
                }))?,
                OutputFormat::Table => print_content_nodes_with_footer(
                    &children.results,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_content_nodes(&children.results, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        PageAction::Copy {
            source_id,
            title,
            space,
            space_id,
            parent,
            root_level,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                if let Some(space) = &space {
                    println!(
                        "Would GET {}/wiki/api/v2/spaces?keys={space}&limit=1 using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/')
                    );
                }
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}?body-format=storage using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    source_id
                );
                println!(
                    "Would POST {}/wiki/api/v2/pages using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let resolved_space_id = resolve_space_id(&client, space.as_deref(), space_id).await?;
            let page = client
                .copy_page(&ConfluencePageCopy {
                    source_id: source_id.clone(),
                    title,
                    space_id: resolved_space_id,
                    parent_id: parent,
                    root_level,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to copy Confluence page `{source_id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_page(&page, global)?;
        }
        PageAction::Update {
            id,
            title,
            parent,
            body,
            body_file,
            representation,
            version,
            message,
            draft,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let endpoint = if body.is_none() && body_file.is_none() && parent.is_none() {
                    format!(
                        "{}/wiki/api/v2/pages/{}/title",
                        profile.instance.trim_end_matches('/'),
                        id
                    )
                } else {
                    format!(
                        "{}/wiki/api/v2/pages/{}",
                        profile.instance.trim_end_matches('/'),
                        id
                    )
                };
                println!("Would PUT {endpoint} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let existing = client.get_page(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence page `{id}` from {}",
                    client.instance_url()
                )
            })?;
            let status = status_from_draft(draft);

            if body.is_none() && body_file.is_none() && parent.is_none() {
                let title = title.ok_or_else(|| {
                    anyhow::anyhow!(
                        "nothing to update; provide --title, --body-file, --body, or --parent"
                    )
                })?;
                let page = client
                    .update_page_title(&id, &title, status)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to update Confluence page title `{id}` from {}",
                            client.instance_url()
                        )
                    })?;
                print_page(&page, global)?;
                return Ok(());
            }

            let (body, representation) = prepare_required_body(
                read_body(body, body_file.as_deref())?,
                representation,
                "page body update and move require --body or --body-file",
            )?;
            let title = title
                .or(existing.title)
                .ok_or_else(|| anyhow::anyhow!("page `{id}` did not include a title"))?;
            let next_version = version
                .or_else(|| {
                    existing
                        .version
                        .as_ref()
                        .and_then(|version| version.number)
                        .map(|number| number + 1)
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("page `{id}` did not include a version; pass --version")
                })?;

            let page = client
                .update_page(&ConfluencePageUpdate {
                    id: id.clone(),
                    status,
                    title,
                    space_id: existing.space_id,
                    parent_id: parent,
                    body,
                    representation,
                    version: next_version,
                    message,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to update Confluence page `{id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_page(&page, global)?;
        }
        PageAction::Delete {
            id,
            purge,
            draft,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/api/v2/pages/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            if !yes {
                anyhow::bail!("refusing to delete page `{id}` without --yes");
            }

            let client = ctx.confluence_client()?;
            client.delete_page(&id, purge, draft).await.map_err(|err| {
                let msg = err.to_string();
                if purge && msg.contains("404") {
                    anyhow::anyhow!(
                        "failed to delete Confluence page `{id}` from {}\n\
                            Hint: to purge a page it must first be in the trash; \
                            run without --purge to move it to trash, then retry with --purge",
                        client.instance_url()
                    )
                } else if msg.contains("404") && !draft {
                    anyhow::anyhow!(
                        "failed to delete Confluence page `{id}` from {}\n\
                            Hint: if this is a draft page, add the `--draft` flag",
                        client.instance_url()
                    )
                } else {
                    anyhow::anyhow!(
                        "failed to delete Confluence page `{id}` from {}: {err}",
                        client.instance_url()
                    )
                }
            })?;
            println!("Deleted Confluence page {id}");
        }
        PageAction::Move { id, parent } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}?body-format=storage using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                println!(
                    "Would PUT {}/wiki/api/v2/pages/{} with parentId `{}` using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id,
                    parent
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let page = client.move_page(&id, &parent).await.with_context(|| {
                format!(
                    "failed to move Confluence page `{id}` under `{parent}` from {}",
                    client.instance_url()
                )
            })?;
            print_page(&page, global)?;
        }
        PageAction::Label { action } => {
            run_page_label(action, global).await?;
        }
        PageAction::Comment { action } => {
            run_page_comment(action, global).await?;
        }
    }

    Ok(())
}
