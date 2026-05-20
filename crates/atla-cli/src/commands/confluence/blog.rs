use anyhow::Context;
use atla_core::{ConfluenceBlogPostCreate, ConfluenceBlogPostSearch, ConfluenceBlogPostUpdate};

use crate::cli::{BlogAction, BlogCommand, GlobalArgs};
use crate::context::AppContext;

use super::blog_comment::run_blog_comment;
use super::blog_label::run_blog_label;
use super::format::{
    confluence_body_representation, print_blog_post, print_blog_posts, print_deleted, read_body,
    resolve_required_space_id, resolve_space_id, status_from_draft,
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

            if global.dry_run {
                if let Some(space) = &space {
                    let space_url = format!(
                        "{}/wiki/api/v2/spaces?keys={space}&limit=1",
                        profile.instance.trim_end_matches('/')
                    );
                    println!("Would GET {space_url} using profile `{profile_name}`");
                }
                println!(
                    "Would POST {}/wiki/api/v2/blogposts using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let body = read_body(body, body_file.as_deref())?;
            let client = ctx.confluence_client()?;
            let space_id = resolve_required_space_id(&client, space.as_deref(), space_id).await?;
            let post = client
                .create_blog_post(&ConfluenceBlogPostCreate {
                    space_id,
                    title,
                    body,
                    representation,
                    status: status_from_draft(draft),
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
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let limit = limit.clamp(1, 250);

            if global.dry_run {
                if let Some(space) = &space {
                    let space_url = format!(
                        "{}/wiki/api/v2/spaces?keys={space}&limit=1",
                        profile.instance.trim_end_matches('/')
                    );
                    println!("Would GET {space_url} using profile `{profile_name}`");
                }

                let mut url = format!(
                    "{}/wiki/api/v2/blogposts?limit={limit}",
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
            let search = ConfluenceBlogPostSearch {
                space_id: resolved_space_id,
                title,
                limit,
            };
            let page = client.list_blog_posts(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence blog posts from {}",
                    client.instance_url()
                )
            })?;

            print_blog_posts(&page.results, global)?;
        }
        BlogAction::View { id, .. } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/blogposts/{}",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let post = client.get_blog_post(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence blog post `{id}` from {}",
                    client.instance_url()
                )
            })?;

            print_blog_post(&post, global)?;
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

            if global.dry_run {
                println!(
                    "Would PUT {}/wiki/api/v2/blogposts/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let existing = client.get_blog_post(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence blog post `{id}` from {}",
                    client.instance_url()
                )
            })?;
            let body = read_body(body, body_file.as_deref())?
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
                    status: status_from_draft(draft),
                    title,
                    space_id: existing.space_id,
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
                    profile.instance.trim_end_matches('/'),
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
