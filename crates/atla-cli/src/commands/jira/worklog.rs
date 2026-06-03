use anyhow::Context;
use atla_core::JiraWorklogCreate;

use crate::cli::{GlobalArgs, IssueWorklogAction, OutputFormat};
use crate::context::AppContext;

use super::format::{print_worklog, print_worklogs, print_worklogs_with_footer};

pub(super) async fn run_issue_worklog(
    action: IssueWorklogAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        IssueWorklogAction::Add {
            key,
            time,
            comment,
            started,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/api/3/issue/{}/worklog using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let worklog = client
                .add_worklog(&JiraWorklogCreate {
                    issue_id_or_key: key.clone(),
                    time_spent: time,
                    comment,
                    started,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to add worklog to Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
            print_worklog(&worklog, global)?;
        }
        IssueWorklogAction::List {
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
                crate::pagination::query_hash("jira.worklog.list", &[("key", key.clone())]);
            let start_at = crate::pagination::decode_jira_offset_token(
                page_token.as_deref(),
                "jira.worklog.list",
                query_hash.clone(),
            )?;

            if global.dry_run {
                println!(
                    "Would GET {}/rest/api/3/issue/{}/worklog?startAt=0&maxResults={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key,
                    max_results
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client
                .list_worklogs_from(&key, max_results, start_at)
                .await
                .with_context(|| {
                    format!(
                        "failed to list worklogs for Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

            let next_start = (!all
                && page
                    .total
                    .is_some_and(|total| start_at + (page.worklogs.len() as u64) < total as u64))
            .then_some(start_at + page.worklogs.len() as u64);
            let next_cli_token = crate::pagination::jira_offset_next_token(
                "jira.worklog.list",
                next_start,
                query_hash,
            )?;
            let next_command = next_cli_token.as_ref().map(|token| {
                crate::pagination::next_command(
                    vec![
                        "atla".to_owned(),
                        "jira".to_owned(),
                        "issue".to_owned(),
                        "worklog".to_owned(),
                        "list".to_owned(),
                        crate::pagination::quote(&key),
                    ],
                    limit,
                    token,
                )
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(
                    &serde_json::json!({"worklogs": page.worklogs, "total": page.total, "pagination": {"isLast": next_cli_token.is_none(), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
                )?,
                OutputFormat::Table => print_worklogs_with_footer(
                    &page,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_worklogs(&page, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
    }

    Ok(())
}
