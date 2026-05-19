use anyhow::Context;
use atla_core::JiraWorklogCreate;

use crate::cli::{GlobalArgs, IssueWorklogAction};
use crate::context::AppContext;

use super::format::{print_worklog, print_worklogs};

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
        IssueWorklogAction::List { key, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let limit = limit.clamp(1, 1000);

            if global.dry_run {
                println!(
                    "Would GET {}/rest/api/3/issue/{}/worklog?startAt=0&maxResults={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key,
                    limit
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client.list_worklogs(&key, limit).await.with_context(|| {
                format!(
                    "failed to list worklogs for Jira issue `{key}` from {}",
                    client.instance_url()
                )
            })?;
            print_worklogs(&page, global)?;
        }
    }

    Ok(())
}
