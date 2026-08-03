use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::Comment { action } = action else {
        unreachable!("issue action dispatcher passed the wrong variant to comment")
    };
    match action {
        IssueCommentAction::Add {
            key,
            body,
            body_flag,
            body_file,
            attachments,
            attachment_mode,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let body =
                read_required_text(body.or(body_flag), body_file.as_deref(), "comment body")?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{}/comment",
                    profile.jira_api_base_url(),
                    key
                );
                for attachment in &attachments {
                    println!(
                        "Would POST {}/rest/api/3/issue/{}/attachments with file `{}` using profile `{profile_name}`",
                        profile.jira_api_base_url(),
                        key,
                        attachment.display()
                    );
                }
                println!("Would POST {url} using profile `{profile_name}`");
                global
                    .output()
                    .print_dry_run_body(&atla_core::jira::comment_request_body(&body))?;
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let mut uploaded = Vec::new();
            for attachment in &attachments {
                uploaded.extend(
                    client
                        .upload_attachment(&key, attachment)
                        .await
                        .with_context(|| {
                            format!(
                                "failed to upload attachment `{}` to Jira issue `{key}` from {}",
                                attachment.display(),
                                client.instance_url()
                            )
                        })?,
                );
            }
            let body = append_jira_attachment_references(&body, &uploaded, attachment_mode);
            let comment = client.add_comment(&key, &body).await.with_context(|| {
                format!(
                    "failed to add comment to Jira issue `{key}` from {}",
                    client.instance_url()
                )
            })?;

            print_comment(&comment, global)?;
        }
        IssueCommentAction::List {
            key,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let max_results = if all { u32::MAX } else { limit.clamp(1, 1000) };
            let query_hash =
                crate::pagination::query_hash("jira.comment.list", &[("key", key.clone())]);
            let start_at = crate::pagination::decode_jira_offset_token(
                page_token.as_deref(),
                "jira.comment.list",
                query_hash.clone(),
            )?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{}/comment?startAt=0&maxResults={max_results}",
                    profile.jira_api_base_url(),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client
                .list_comments_from(&key, max_results, start_at)
                .await
                .with_context(|| {
                    format!(
                        "failed to list comments for Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

            let next_start = (!all
                && page
                    .total
                    .is_some_and(|total| start_at + (page.comments.len() as u64) < total as u64))
            .then_some(start_at + page.comments.len() as u64);
            let next_cli_token = crate::pagination::jira_offset_next_token(
                "jira.comment.list",
                next_start,
                query_hash,
            )?;
            let next_command = next_cli_token.as_ref().map(|token| {
                crate::pagination::next_command(
                    vec![
                        "atla".to_owned(),
                        "jira".to_owned(),
                        "issue".to_owned(),
                        "comment".to_owned(),
                        "list".to_owned(),
                        crate::pagination::quote(&key),
                    ],
                    limit,
                    token,
                )
            });
            match global.output.unwrap_or(crate::cli::OutputFormat::Table) {
                        crate::cli::OutputFormat::Json => global.output().print_json(
                            &serde_json::json!({"comments": page.comments, "total": page.total, "pagination": {"isLast": next_cli_token.is_none(), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
                        )?,
                        crate::cli::OutputFormat::Table => print_comments_with_footer(
                            &page,
                            global,
                            next_command
                                .as_deref()
                                .map(crate::pagination::next_page_footer),
                        )?,
                        crate::cli::OutputFormat::Csv | crate::cli::OutputFormat::Keys => {
                            print_comments(&page, global)?;
                            if let Some(command) = next_command {
                                eprintln!("{}", crate::pagination::next_page_footer(&command));
                            }
                        }
                    }
        }
        IssueCommentAction::Update {
            key,
            comment_id,
            body,
            body_file,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let body = read_required_text(body, body_file.as_deref(), "comment body")?;

            if global.dry_run {
                println!(
                    "Would PUT {}/rest/api/3/issue/{}/comment/{} using profile `{profile_name}`",
                    profile.jira_api_base_url(),
                    key,
                    comment_id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let comment = client
                .update_comment(&key, &comment_id, &body)
                .await
                .with_context(|| {
                    format!(
                        "failed to update comment `{comment_id}` on Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

            print_comment(&comment, global)?;
        }
        IssueCommentAction::Delete {
            key,
            comment_id,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if !yes && !global.dry_run {
                anyhow::bail!("refusing to delete Jira comment `{comment_id}` without --yes");
            }

            if global.dry_run {
                println!(
                    "Would DELETE {}/rest/api/3/issue/{}/comment/{} using profile `{profile_name}`",
                    profile.jira_api_base_url(),
                    key,
                    comment_id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .delete_comment(&key, &comment_id)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete comment `{comment_id}` on Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("comment", &comment_id, global)?;
        }
    };
    Ok(())
}
