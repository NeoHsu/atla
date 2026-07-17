use anyhow::Context;
use atla_core::{
    ConfluenceAttachmentSearch, ConfluenceContentTreeSearch, ConfluencePageCopy,
    ConfluencePageCreate, ConfluencePageSearch, ConfluencePageTitleUpdate, ConfluencePageUpdate,
    markdown::AdfToMarkdownOptions,
};

use crate::cli::{ContentViewFormat, GlobalArgs, OutputFormat, PageAction, PageCommand};
use crate::context::AppContext;

use super::format::{
    markdown_to_adf_options_for_body, open_web_url, prepare_optional_body_with_options,
    prepare_required_body_with_options, print_content_nodes, print_content_nodes_with_footer,
    print_deleted, print_page, print_page_body_view, print_page_with_attachments, print_pages,
    print_pages_with_footer, read_body, resolve_required_space_id, resolve_space_id,
    status_from_draft, view_format_body_representation,
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
            numbered_table_rows,
            mentions,
            resolve_mentions,
            draft,
            private,
            root_level,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run
                && let Some(path) = body_file.as_deref()
            {
                crate::output::register_plan_input(path)?;
            }
            let body = read_body(body, body_file.as_deref())?;
            if global.dry_run && resolve_mentions {
                anyhow::bail!(
                    "--dry-run cannot resolve mentions without network access; pass explicit --mention values or omit --resolve-mentions"
                );
            }
            let jira_client = if resolve_mentions {
                Some(ctx.jira_client()?)
            } else {
                None
            };
            let markdown_options = markdown_to_adf_options_for_body(
                body.as_deref(),
                representation,
                numbered_table_rows,
                &mentions,
                resolve_mentions,
                jira_client.as_ref(),
            )
            .await?;
            let (body, representation) =
                prepare_optional_body_with_options(body, representation, markdown_options)?;
            let status = status_from_draft(draft);

            if global.dry_run {
                let space_id = space_id.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--dry-run requires --space-id because --space cannot be resolved without network access"
                    )
                })?;
                let request = ConfluencePageCreate {
                    space_id,
                    title,
                    parent_id: parent,
                    body,
                    representation,
                    status,
                    private: private.then_some(true),
                    root_level: root_level.then_some(true),
                };
                let mut endpoint =
                    format!("{}/wiki/api/v2/pages", profile.confluence_api_base_url());
                let mut query = Vec::new();
                if private {
                    query.push("private=true");
                }
                if root_level {
                    query.push("root-level=true");
                }
                if !query.is_empty() {
                    endpoint.push('?');
                    endpoint.push_str(&query.join("&"));
                }
                let body = request.request_body();
                if global.output == Some(crate::cli::OutputFormat::Json) {
                    crate::output::print_operation_plan(
                        "confluence.page.create",
                        profile_name,
                        &profile.instance,
                        "POST",
                        endpoint,
                        Some(body),
                        vec!["spaceId supplied explicitly".to_owned()],
                        Vec::new(),
                    )?;
                } else {
                    println!("Would POST {endpoint} using profile `{profile_name}`");
                    crate::output::print_dry_run_body(&body)?;
                }
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let space_id = resolve_required_space_id(&client, space.as_deref(), space_id).await?;
            let page = client
                .create_page(&ConfluencePageCreate {
                    space_id,
                    title,
                    parent_id: parent,
                    body,
                    representation,
                    status,
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
                        profile.confluence_api_base_url()
                    );
                    println!("Would GET {space_url} using profile `{profile_name}`");
                }

                let mut url = format!(
                    "{}/wiki/api/v2/pages?limit={limit}",
                    profile.confluence_api_base_url()
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
            preserve_table_options,
            with_attachments,
        } => {
            if preserve_table_options && !matches!(format, Some(ContentViewFormat::Markdown)) {
                anyhow::bail!("--preserve-table-options requires --format markdown");
            }
            let markdown_options = AdfToMarkdownOptions {
                table_numbered_rows_directives: preserve_table_options,
            };
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/pages/{}",
                    profile.confluence_api_base_url(),
                    id
                );
                println!("Would GET {url} using profile `{profile_name}`");
                if with_attachments {
                    let att_url = format!(
                        "{}/wiki/api/v2/pages/{}/attachments",
                        profile.confluence_api_base_url(),
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

            let attachments = if with_attachments {
                let search = ConfluenceAttachmentSearch {
                    page_id: id.clone(),
                    filename: None,
                    limit: 250,
                    cursor: None,
                };
                Some(
                    client
                        .list_page_attachments(&search)
                        .await
                        .with_context(|| format!("failed to list attachments for page `{id}`"))?
                        .results,
                )
            } else {
                None
            };
            if let Some(format) = format {
                print_page_body_view(
                    &page,
                    format,
                    attachments.as_deref(),
                    global,
                    markdown_options,
                )?;
            } else if let Some(attachments) = attachments {
                print_page_with_attachments(&page, &attachments, global)?;
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
                        profile.confluence_api_base_url(),
                        search.page_id,
                        search.limit
                    )
                } else {
                    format!(
                        "{}/wiki/api/v2/pages/{}/direct-children?limit={}",
                        profile.confluence_api_base_url(),
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
                        profile.confluence_api_base_url()
                    );
                }
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}?body-format=storage using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
                    source_id
                );
                println!(
                    "Would POST {}/wiki/api/v2/pages using profile `{profile_name}`",
                    profile.confluence_api_base_url()
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
            numbered_table_rows,
            mentions,
            resolve_mentions,
            version,
            message,
            draft,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            let status = status_from_draft(draft);

            if body.is_none() && body_file.is_none() && parent.is_none() {
                let title = title.ok_or_else(|| {
                    anyhow::anyhow!(
                        "nothing to update; provide --title, --body-file, --body, or --parent"
                    )
                })?;
                let request = ConfluencePageTitleUpdate { title, status };
                if global.dry_run {
                    let url = format!(
                        "{}/wiki/api/v2/pages/{}/title",
                        profile.confluence_api_base_url(),
                        id
                    );
                    let body = request.request_body();
                    if global.output == Some(crate::cli::OutputFormat::Json) {
                        crate::output::print_operation_plan(
                            "confluence.page.update",
                            profile_name,
                            &profile.instance,
                            "PUT",
                            url,
                            Some(body),
                            vec!["title supplied explicitly".to_owned()],
                            Vec::new(),
                        )?;
                    } else {
                        println!("Would PUT {url} using profile `{profile_name}`");
                        crate::output::print_dry_run_body(&body)?;
                    }
                    return Ok(());
                }

                let client = ctx.confluence_client()?;
                let page = client
                    .update_page_title(&id, &request.title, request.status)
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

            if global.dry_run
                && let Some(path) = body_file.as_deref()
            {
                crate::output::register_plan_input(path)?;
            }
            let body = read_body(body, body_file.as_deref())?;
            if global.dry_run && resolve_mentions {
                anyhow::bail!(
                    "--dry-run cannot resolve mentions without network access; pass explicit --mention values or omit --resolve-mentions"
                );
            }
            let jira_client = if resolve_mentions {
                Some(ctx.jira_client()?)
            } else {
                None
            };
            let markdown_options = markdown_to_adf_options_for_body(
                body.as_deref(),
                representation,
                numbered_table_rows,
                &mentions,
                resolve_mentions,
                jira_client.as_ref(),
            )
            .await?;
            let (body, representation) = prepare_required_body_with_options(
                body,
                representation,
                markdown_options,
                "page body update and move require --body or --body-file",
            )?;

            if global.dry_run {
                let title = title.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--dry-run requires --title for a full page update because the current title cannot be loaded without network access"
                    )
                })?;
                let version = version.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--dry-run requires --version for a full page update because the next version cannot be loaded without network access"
                    )
                })?;
                let request = ConfluencePageUpdate {
                    id: id.clone(),
                    status,
                    title,
                    space_id: None,
                    parent_id: parent,
                    body,
                    representation,
                    version,
                    message,
                };
                let url = format!(
                    "{}/wiki/api/v2/pages/{}",
                    profile.confluence_api_base_url(),
                    id
                );
                let body = request.request_body();
                if global.output == Some(crate::cli::OutputFormat::Json) {
                    crate::output::print_operation_plan(
                        "confluence.page.update",
                        profile_name,
                        &profile.instance,
                        "PUT",
                        url,
                        Some(body),
                        vec![
                            "title supplied explicitly".to_owned(),
                            "version supplied explicitly".to_owned(),
                        ],
                        Vec::new(),
                    )?;
                } else {
                    println!("Would PUT {url} using profile `{profile_name}`");
                    crate::output::print_dry_run_body(&body)?;
                }
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let existing = client.get_page(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence page `{id}` from {}",
                    client.instance_url()
                )
            })?;
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
                    space_id: None,
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
                    profile.confluence_api_base_url(),
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
            print_deleted("page", &id, global)?;
        }
        PageAction::Move { id, parent } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}?body-format=storage using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
                    id
                );
                println!(
                    "Would PUT {}/wiki/api/v2/pages/{} with parentId `{}` using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
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
