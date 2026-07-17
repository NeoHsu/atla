use anyhow::Context;
use atla_core::{ConfluenceAttachmentSearch, ConfluenceAttachmentUpload};

use crate::cli::{AttachmentAction, AttachmentCommand, GlobalArgs, OutputFormat};
use crate::context::AppContext;

use super::format::{
    print_attachment, print_attachment_download, print_attachments, print_attachments_with_footer,
    print_deleted,
};

pub(super) async fn run_attachment(
    command: AttachmentCommand,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match command.action {
        AttachmentAction::List {
            page_id,
            filename,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let filename = filename.filter(|s| !s.is_empty());
            let query_hash = crate::pagination::query_hash(
                "confluence.attachment.list",
                &[
                    ("pageId", page_id.clone()),
                    ("filename", filename.clone().unwrap_or_default()),
                ],
            );
            let cursor = crate::pagination::decode_confluence_cursor_token(
                page_token.as_deref(),
                "confluence.attachment.list",
                query_hash.clone(),
            )?;
            let search = ConfluenceAttachmentSearch {
                page_id,
                filename,
                limit: if all { u32::MAX } else { limit.clamp(1, 250) },
                cursor,
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}/attachments?limit={} using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
                    search.page_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let page = client
                .list_page_attachments(&search)
                .await
                .with_context(|| {
                    format!(
                        "failed to list Confluence page attachments from {}",
                        client.instance_url()
                    )
                })?;

            let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
                crate::pagination::confluence_cursor_next_token(
                    "confluence.attachment.list",
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
                    "attachment".to_owned(),
                    "list".to_owned(),
                    crate::pagination::quote(&search.page_id),
                ];
                if let Some(filename) = &search.filename {
                    parts.push("--filename".to_owned());
                    parts.push(crate::pagination::quote(filename));
                }
                crate::pagination::next_command(parts, limit, token)
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(&serde_json::json!({
                    "results": page.results,
                    "pagination": { "isLast": page.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command }
                }))?,
                OutputFormat::Table => print_attachments_with_footer(
                    &page.results,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_attachments(&page.results, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        AttachmentAction::View { attachment_id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/attachments/{} using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
                    attachment_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let attachment = client
                .get_attachment(&attachment_id)
                .await
                .with_context(|| {
                    format!(
                        "failed to load Confluence attachment `{attachment_id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_attachment(&attachment, global)?;
        }
        AttachmentAction::Upload {
            page_id,
            file,
            comment,
            minor_edit,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let upload = ConfluenceAttachmentUpload {
                page_id,
                file,
                comment,
                minor_edit,
            };

            if global.dry_run {
                println!(
                    "Would PUT {}/wiki/rest/api/content/{}/child/attachment with file `{}` using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
                    upload.page_id,
                    upload.file.display()
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let page = client
                .upload_page_attachment(&upload)
                .await
                .with_context(|| {
                    format!(
                        "failed to upload Confluence page attachment to {}",
                        client.instance_url()
                    )
                })?;

            print_attachments(&page.results, global)?;
        }
        AttachmentAction::Download {
            attachment_id,
            save_to,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/attachments/{} then download its file using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
                    attachment_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let download = client
                .download_attachment(&attachment_id, save_to.as_deref())
                .await
                .with_context(|| {
                    format!(
                        "failed to download Confluence attachment `{attachment_id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_attachment_download(
                &download.path.display().to_string(),
                download.bytes,
                global,
            )?;
        }
        AttachmentAction::Delete {
            attachment_id,
            purge,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/api/v2/attachments/{} using profile `{profile_name}`",
                    profile.confluence_api_base_url(),
                    attachment_id
                );
                return Ok(());
            }
            if !yes {
                anyhow::bail!("refusing to delete attachment `{attachment_id}` without --yes");
            }

            let client = ctx.confluence_client()?;
            client
                .delete_attachment(&attachment_id, purge)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete Confluence attachment `{attachment_id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("attachment", &attachment_id, global)?;
        }
    }

    Ok(())
}
