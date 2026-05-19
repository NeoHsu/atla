use anyhow::Context;
use atla_core::{JiraSprintCreate, JiraSprintSearch, JiraSprintUpdate};

use crate::cli::{GlobalArgs, SprintAction, SprintCommand};
use crate::context::AppContext;

use super::format::{
    parse_issue_fields, print_issues, print_sprint, print_sprint_issue_move, print_sprints,
};

pub(super) async fn run_sprint(command: SprintCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        SprintAction::List {
            board,
            state,
            limit,
        } => {
            run_sprint_list(board, state, limit, global).await?;
        }
        SprintAction::Active { board, limit } => {
            run_sprint_list(board, Some("active".to_owned()), limit, global).await?;
        }
        SprintAction::View { id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/rest/agile/1.0/sprint/{id}",
                    profile.instance.trim_end_matches('/')
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let sprint = client.get_sprint(id).await.with_context(|| {
                format!(
                    "failed to load Jira sprint `{id}` from {}",
                    client.instance_url()
                )
            })?;

            print_sprint(&sprint, global)?;
        }
        SprintAction::Create {
            board,
            name,
            start,
            end,
            goal,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/agile/1.0/sprint using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let sprint = client
                .create_sprint(&JiraSprintCreate {
                    board_id: board,
                    name,
                    start_date: start,
                    end_date: end,
                    goal,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to create Jira sprint from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint(&sprint, global)?;
        }
        SprintAction::Start { id, start, end } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would PUT {}/rest/agile/1.0/sprint/{} with state active using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let existing = client.get_sprint(id).await.with_context(|| {
                format!(
                    "failed to load Jira sprint `{id}` from {}",
                    client.instance_url()
                )
            })?;
            if existing.state.as_deref() == Some("active") {
                anyhow::bail!("sprint `{id}` is already active");
            }
            let sprint = client
                .update_sprint(&JiraSprintUpdate {
                    id,
                    state: Some("active".to_owned()),
                    name: None,
                    start_date: start,
                    end_date: end,
                    goal: None,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to start Jira sprint `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint(&sprint, global)?;
        }
        SprintAction::Close { id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would PUT {}/rest/agile/1.0/sprint/{} with state closed using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let sprint = client
                .update_sprint(&JiraSprintUpdate {
                    id,
                    state: Some("closed".to_owned()),
                    name: None,
                    start_date: None,
                    end_date: None,
                    goal: None,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to close Jira sprint `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint(&sprint, global)?;
        }
        SprintAction::Add { id, issues } => {
            if issues.is_empty() {
                anyhow::bail!("provide at least one issue with --issues");
            }
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/agile/1.0/sprint/{}/issue using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .move_issues_to_sprint(id, &issues)
                .await
                .with_context(|| {
                    format!(
                        "failed to move issues to Jira sprint `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint_issue_move(id, &issues, global)?;
        }
        SprintAction::Remove { id, issues } => {
            if issues.is_empty() {
                anyhow::bail!("provide at least one issue with --issues");
            }
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/agile/1.0/backlog/issue for sprint `{id}` using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .move_issues_to_backlog(&issues)
                .await
                .with_context(|| {
                    format!(
                        "failed to remove issues from Jira sprint `{id}` via backlog move from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint_issue_move(id, &issues, global)?;
        }
        SprintAction::Issues { id, limit, fields } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let requested_fields = parse_issue_fields(fields.as_deref())?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/agile/1.0/sprint/{id}/issue",
                    profile.instance.trim_end_matches('/')
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client
                .get_sprint_issues(id, limit, requested_fields.clone())
                .await
                .with_context(|| {
                    format!(
                        "failed to list issues for Jira sprint `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_issues(&page.issues, global, requested_fields.as_deref())?;
        }
    }

    Ok(())
}

pub(super) async fn run_sprint_list(
    board_id: u64,
    state: Option<String>,
    limit: u32,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let search = JiraSprintSearch {
        board_id,
        start_at: 0,
        max_results: limit.clamp(1, 1000),
        state,
    };

    if global.dry_run {
        let url = format!(
            "{}/rest/agile/1.0/board/{}/sprint?startAt={}&maxResults={}",
            profile.instance.trim_end_matches('/'),
            search.board_id,
            search.start_at,
            search.max_results
        );
        if let Some(state) = &search.state {
            println!("Would GET {url} with state `{state}` using profile `{profile_name}`");
        } else {
            println!("Would GET {url} using profile `{profile_name}`");
        }
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let page = client.list_sprints(&search).await.with_context(|| {
        format!(
            "failed to list Jira sprints for board `{board_id}` from {}",
            client.instance_url()
        )
    })?;

    print_sprints(&page, global)?;
    Ok(())
}
