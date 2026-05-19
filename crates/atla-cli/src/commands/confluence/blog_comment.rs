use anyhow::Context;
use atla_core::{ConfluenceCommentCreate, ConfluenceCommentSearch};

use crate::cli::{BlogCommentAction, GlobalArgs};
use crate::context::AppContext;

use super::format::{confluence_body_representation, print_comment, print_comments, read_body};

pub(super) async fn run_blog_comment(
    action: BlogCommentAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        BlogCommentAction::List { blog_id, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceCommentSearch {
                content_id: blog_id,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/blogposts/{}/footer-comments?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.content_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let comments = client.list_blog_comments(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence blog comments from {}",
                    client.instance_url()
                )
            })?;
            print_comments(&comments, global)?;
        }
        BlogCommentAction::Add {
            blog_id,
            body,
            body_file,
            parent,
            representation,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let body = read_body(body, body_file.as_deref())?
                .ok_or_else(|| anyhow::anyhow!("missing comment body"))?;
            let representation = confluence_body_representation(representation)?;

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/api/v2/footer-comments using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let comment = client
                .add_blog_comment(&ConfluenceCommentCreate {
                    content_id: blog_id,
                    parent_comment_id: parent,
                    body,
                    representation,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to add Confluence blog comment from {}",
                        client.instance_url()
                    )
                })?;
            print_comment(&comment, global)?;
        }
    }

    Ok(())
}
