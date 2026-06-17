use anyhow::Context;
use atla_core::{ConfluenceCommentCreate, ConfluenceCommentSearch, markdown::MarkdownToAdfOptions};

use crate::cli::{GlobalArgs, OutputFormat, PageCommentAction};
use crate::context::AppContext;

use super::format::{
    prepare_required_body_with_options, print_comment, print_comments, print_comments_with_footer,
    print_deleted, read_body,
};

pub(super) async fn run_page_comment(
    action: PageCommentAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        PageCommentAction::List {
            page_id,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let query_hash = crate::pagination::query_hash(
                "confluence.page.comment.list",
                &[("pageId", page_id.clone())],
            );
            let cursor = crate::pagination::decode_confluence_cursor_token(
                page_token.as_deref(),
                "confluence.page.comment.list",
                query_hash.clone(),
            )?;
            let search = ConfluenceCommentSearch {
                content_id: page_id,
                limit: if all { u32::MAX } else { limit.clamp(1, 250) },
                cursor,
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

            let next_cli_token = if !all && matches!(comments.is_last, Some(false)) {
                crate::pagination::confluence_cursor_next_token(
                    "confluence.page.comment.list",
                    comments.next_cursor.clone(),
                    query_hash,
                )?
            } else {
                None
            };
            let next_command = next_cli_token.as_ref().map(|token| {
                crate::pagination::next_command(
                    vec![
                        "atla".to_owned(),
                        "confluence".to_owned(),
                        "page".to_owned(),
                        "comment".to_owned(),
                        "list".to_owned(),
                        crate::pagination::quote(&search.content_id),
                    ],
                    limit,
                    token,
                )
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(
                    &serde_json::json!({"results": comments.results, "pagination": {"isLast": comments.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
                )?,
                OutputFormat::Table => print_comments_with_footer(
                    &comments,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_comments(&comments, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        PageCommentAction::Add {
            page_id,
            body,
            body_file,
            parent,
            representation,
            numbered_table_rows,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let (body, representation) = prepare_required_body_with_options(
                read_body(body, body_file.as_deref())?,
                representation,
                MarkdownToAdfOptions {
                    numbered_table_rows,
                },
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
