use anyhow::Context;
use atla_core::{ConfluenceBlogPostCreate, ConfluenceBlogPostSearch, ConfluenceBlogPostUpdate};

use crate::cli::{BlogAction, BlogCommand, GlobalArgs, OutputFormat};
use crate::context::AppContext;

use super::blog_comment::run_blog_comment;
use super::blog_label::run_blog_label;
use super::format::{
    confluence_body_representation, print_blog_body_view, print_blog_post, print_blog_posts,
    print_blog_posts_with_footer, print_deleted, read_body, resolve_required_space_id,
    resolve_space_id, status_from_draft, view_format_body_representation,
};

pub(super) async fn run_blog(command: BlogCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        BlogAction::Create {
            space,
            space_id,
            title,
            body,
            body_file,
            representation,
            draft,
            private,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let representation = confluence_body_representation(representation)?;

            if global.dry_run
                && let Some(path) = body_file.as_deref()
            {
                crate::output::register_plan_input(path)?;
            }
            let body = read_body(body, body_file.as_deref())?;
            let status = status_from_draft(draft);

            if global.dry_run {
                let space_id = space_id.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--dry-run requires --space-id because --space cannot be resolved without network access"
                    )
                })?;
                let request = ConfluenceBlogPostCreate {
                    space_id,
                    title,
                    body,
                    representation,
                    status,
                    private: private.then_some(true),
                };
                let endpoint = format!(
                    "{}/wiki/api/v2/blogposts{}",
                    profile.confluence_api_base_url(),
                    if private { "?private=true" } else { "" }
                );
                let body = request.request_body();
                if global.output == Some(OutputFormat::Json) {
                    crate::output::print_operation_plan(
                        "confluence.blog.create",
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
            let post = client
                .create_blog_post(&ConfluenceBlogPostCreate {
                    space_id,
                    title,
                    body,
                    representation,
                    status,
                    private: private.then_some(true),
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to create Confluence blog post in {}",
                        client.instance_url()
                    )
                })?;

            print_blog_post(&post, global)?;
        }
        BlogAction::List {
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
                    "{}/wiki/api/v2/blogposts?limit={limit}",
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
            let query_hash = crate::pagination::query_hash(
                "confluence.blog.list",
                &[
                    ("spaceId", resolved_space_id.clone().unwrap_or_default()),
                    ("title", title.clone().unwrap_or_default()),
                ],
            );
            let cursor = crate::pagination::decode_confluence_cursor_token(
                page_token.as_deref(),
                "confluence.blog.list",
                query_hash.clone(),
            )?;
            let search = ConfluenceBlogPostSearch {
                space_id: resolved_space_id,
                title,
                limit,
                cursor,
            };
            let page = client.list_blog_posts(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence blog posts from {}",
                    client.instance_url()
                )
            })?;

            let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
                crate::pagination::confluence_cursor_next_token(
                    "confluence.blog.list",
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
                    "blog".to_owned(),
                    "list".to_owned(),
                ];
                if let Some(space_id) = &search.space_id {
                    parts.push("--space-id".to_owned());
                    parts.push(crate::pagination::quote(space_id));
                }
                if let Some(title) = &search.title {
                    parts.push("--title".to_owned());
                    parts.push(crate::pagination::quote(title));
                }
                crate::pagination::next_command(parts, limit, token)
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(&serde_json::json!({
                    "results": page.results,
                    "pagination": { "isLast": page.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command }
                }))?,
                OutputFormat::Table => print_blog_posts_with_footer(
                    &page.results,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_blog_posts(&page.results, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        BlogAction::View { id, format } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/blogposts/{}",
                    profile.confluence_api_base_url(),
                    id
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let post = client
                .get_blog_post_with_body_format(
                    &id,
                    format.and_then(view_format_body_representation),
                )
                .await
                .with_context(|| {
                    format!(
                        "failed to load Confluence blog post `{id}` from {}",
                        client.instance_url()
                    )
                })?;

            if let Some(format) = format {
                print_blog_body_view(&post, format, global)?;
            } else {
                print_blog_post(&post, global)?;
            }
        }
        BlogAction::Update {
            id,
            title,
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
            let representation = confluence_body_representation(representation)?;

            if global.dry_run
                && let Some(path) = body_file.as_deref()
            {
                crate::output::register_plan_input(path)?;
            }
            let body = read_body(body, body_file.as_deref())?;
            let status = status_from_draft(draft);

            if global.dry_run {
                let body = body.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--dry-run requires --body or --body-file because the current body cannot be loaded without network access"
                    )
                })?;
                let title = title.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--dry-run requires --title because the current title cannot be loaded without network access"
                    )
                })?;
                let version = version.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--dry-run requires --version because the next version cannot be loaded without network access"
                    )
                })?;
                let request = ConfluenceBlogPostUpdate {
                    id: id.clone(),
                    status,
                    title,
                    space_id: None,
                    body,
                    representation,
                    version,
                    message,
                };
                let url = format!(
                    "{}/wiki/api/v2/blogposts/{}",
                    profile.confluence_api_base_url(),
                    id
                );
                let body = request.request_body();
                if global.output == Some(OutputFormat::Json) {
                    crate::output::print_operation_plan(
                        "confluence.blog.update",
                        profile_name,
                        &profile.instance,
                        "PUT",
                        url,
                        Some(body),
                        vec![
                            "body supplied explicitly".to_owned(),
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
            let existing = client.get_blog_post(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence blog post `{id}` from {}",
                    client.instance_url()
                )
            })?;
            let body = body
                .or_else(|| existing.body.clone())
                .ok_or_else(|| anyhow::anyhow!("provide --body or --body-file"))?;
            let title = title
                .or(existing.title)
                .ok_or_else(|| anyhow::anyhow!("blog post `{id}` did not include a title"))?;
            let next_version = version
                .or_else(|| {
                    existing
                        .version
                        .as_ref()
                        .and_then(|version| version.number)
                        .map(|number| number + 1)
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("blog post `{id}` did not include a version; pass --version")
                })?;
            let post = client
                .update_blog_post(&ConfluenceBlogPostUpdate {
                    id: id.clone(),
                    status,
                    title,
                    space_id: None,
                    body,
                    representation,
                    version: next_version,
                    message,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to update Confluence blog post `{id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_blog_post(&post, global)?;
        }
        BlogAction::Delete {
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
                    "Would DELETE {}/wiki/api/v2/blogposts/{} using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
                    id
                );
                return Ok(());
            }
            if !yes {
                anyhow::bail!("refusing to delete blog post `{id}` without --yes");
            }

            let client = ctx.confluence_client()?;
            client
                .delete_blog_post(&id, purge, draft)
                .await
                .map_err(|err| {
                    let msg = err.to_string();
                    if purge && msg.contains("404") {
                        anyhow::anyhow!(
                            "failed to delete Confluence blog post `{id}` from {}\n\
                            Hint: to purge a blog post it must first be in the trash; \
                            run without --purge to move it to trash, then retry with --purge",
                            client.instance_url()
                        )
                    } else if msg.contains("404") && !draft {
                        anyhow::anyhow!(
                            "failed to delete Confluence blog post `{id}` from {}\n\
                            Hint: if this is a draft blog post, add the `--draft` flag",
                            client.instance_url()
                        )
                    } else {
                        anyhow::anyhow!(
                            "failed to delete Confluence blog post `{id}` from {}: {err}",
                            client.instance_url()
                        )
                    }
                })?;
            print_deleted("blogPost", &id, global)?;
        }
        BlogAction::Label { action } => {
            run_blog_label(action, global).await?;
        }
        BlogAction::Comment { action } => {
            run_blog_comment(action, global).await?;
        }
    }

    Ok(())
}
