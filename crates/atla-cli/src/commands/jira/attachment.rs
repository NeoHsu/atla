use anyhow::Context;

use crate::cli::{GlobalArgs, IssueAttachmentAction, OutputFormat};
use crate::context::AppContext;
use crate::output;

use super::format::{print_attachment_downloads, print_attachments};

pub(super) async fn run_issue_attachment(
    action: IssueAttachmentAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        IssueAttachmentAction::Upload { key, file } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{key}/attachments",
                    profile.instance.trim_end_matches('/')
                );
                println!("Would POST {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let attachments = client
                .upload_attachment(&key, &file)
                .await
                .with_context(|| {
                    format!(
                        "failed to upload attachment to Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

            print_attachments(&attachments, global)?;
        }
        IssueAttachmentAction::List { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{key}?fields=attachment",
                    profile.instance.trim_end_matches('/')
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let attachments = client.list_issue_attachments(&key).await.with_context(|| {
                format!(
                    "failed to list attachments for Jira issue `{key}` from {}",
                    client.instance_url()
                )
            })?;

            print_attachments(&attachments, global)?;
        }
        IssueAttachmentAction::Download { target, all, dest } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                if all {
                    println!(
                        "Would GET {}/rest/api/3/issue/{}?fields=attachment, then download each attachment using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/'),
                        target
                    );
                } else {
                    println!(
                        "Would GET {}/rest/api/3/attachment/{}, then download its content using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/'),
                        target
                    );
                }
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let downloads = if all {
                client
                    .download_issue_attachments(&target, dest.as_deref())
                    .await
                    .with_context(|| {
                        format!(
                            "failed to download Jira issue attachments for `{target}` from {}",
                            client.instance_url()
                        )
                    })?
            } else {
                vec![
                    client
                        .download_attachment(&target, dest.as_deref())
                        .await
                        .with_context(|| {
                            format!(
                                "failed to download Jira attachment `{target}` from {}",
                                client.instance_url()
                            )
                        })?,
                ]
            };

            print_attachment_downloads(&downloads, global)?;
        }
        IssueAttachmentAction::Delete { attachment_id, yes } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if !yes && !global.dry_run {
                anyhow::bail!("refusing to delete attachment `{attachment_id}` without --yes");
            }

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/attachment/{attachment_id}",
                    profile.instance.trim_end_matches('/')
                );
                println!("Would DELETE {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .delete_attachment(&attachment_id)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete attachment `{attachment_id}` from {}",
                        client.instance_url()
                    )
                })?;

            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => {
                    output::print_json(&serde_json::json!({ "deleted": attachment_id }))
                }
                OutputFormat::Keys => {
                    println!("{attachment_id}");
                    Ok(())
                }
                OutputFormat::Csv => {
                    println!("{},true", output::csv_cell(&attachment_id));
                    Ok(())
                }
                OutputFormat::Table => {
                    println!("Deleted: {attachment_id}");
                    Ok(())
                }
            }?;
        }
    }

    Ok(())
}
