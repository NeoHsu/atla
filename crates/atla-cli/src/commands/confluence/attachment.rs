use anyhow::Context;
use atla_core::{ConfluenceAttachmentSearch, ConfluenceAttachmentUpload};

use crate::cli::{AttachmentAction, AttachmentCommand, GlobalArgs};
use crate::context::AppContext;

use super::format::{
    print_attachment, print_attachment_download, print_attachments, print_deleted,
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
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceAttachmentSearch {
                page_id,
                filename: filename.filter(|s| !s.is_empty()),
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}/attachments?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
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

            print_attachments(&page.results, global)?;
        }
        AttachmentAction::View { attachment_id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/attachments/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
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
                    profile.instance.trim_end_matches('/'),
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
            output,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/attachments/{} then download its file using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    attachment_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let download = client
                .download_attachment(&attachment_id, output.as_deref())
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
                    profile.instance.trim_end_matches('/'),
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
