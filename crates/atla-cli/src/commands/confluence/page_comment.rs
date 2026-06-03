use anyhow::Context;
use atla_core::{ConfluenceCommentCreate, ConfluenceCommentSearch};

use crate::cli::{GlobalArgs, PageCommentAction};
use crate::context::AppContext;

use super::format::{
    prepare_required_body, print_comment, print_comments, print_deleted, read_body,
};

pub(super) async fn run_page_comment(
    action: PageCommentAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        PageCommentAction::List { page_id, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceCommentSearch {
                content_id: page_id,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}/footer-comments?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.content_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let comments = client.list_page_comments(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence page comments from {}",
                    client.instance_url()
                )
            })?;

            crate::output::warn_if_truncated(
                matches!(comments.is_last, Some(false)),
                comments.results.len(),
                "comments",
            );

            print_comments(&comments, global)?;
        }
        PageCommentAction::Add {
            page_id,
            body,
            body_file,
            parent,
            representation,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let (body, representation) = prepare_required_body(
                read_body(body, body_file.as_deref())?,
                representation,
                "missing comment body",
            )?;

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/api/v2/footer-comments using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let comment = client
                .add_page_comment(&ConfluenceCommentCreate {
                    content_id: page_id,
                    parent_comment_id: parent,
                    body,
                    representation,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to add Confluence page comment from {}",
                        client.instance_url()
                    )
                })?;
            print_comment(&comment, global)?;
        }
        PageCommentAction::Delete {
            page_id,
            comment_id,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if !yes && !global.dry_run {
                anyhow::bail!("refusing to delete Confluence comment `{comment_id}` without --yes");
            }

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/api/v2/footer-comments/{} for page `{page_id}` using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    comment_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            client
                .delete_page_comment(&comment_id)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete Confluence comment `{comment_id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("comment", &comment_id, global)?;
        }
    }

    Ok(())
}
