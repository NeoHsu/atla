use anyhow::Context;
use atla_core::{JiraSprintCreate, JiraSprintSearch, JiraSprintUpdate};

use crate::cli::{GlobalArgs, OutputFormat, SprintAction, SprintCommand};
use crate::context::AppContext;

use super::format::{
    parse_issue_fields, print_issues, print_issues_with_footer, print_sprint,
    print_sprint_issue_move, print_sprints, print_sprints_with_footer,
};

pub(super) async fn run_sprint(command: SprintCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        SprintAction::List {
            board,
            state,
            limit,
            all,
            page_token,
        } => {
            run_sprint_list(board, state, limit, all, page_token, global).await?;
        }
        SprintAction::Active {
            board,
            limit,
            all,
            page_token,
        } => {
            run_sprint_list(
                board,
                Some("active".to_owned()),
                limit,
                all,
                page_token,
                global,
            )
            .await?;
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
        SprintAction::Issues {
            id,
            limit,
            all,
            fields,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let requested_fields = parse_issue_fields(fields.as_deref())?;
            let max_results = if all { u32::MAX } else { limit };
            let query_hash = crate::pagination::query_hash(
                "jira.sprint.issues",
                &[
                    ("id", id.to_string()),
                    (
                        "fields",
                        requested_fields.clone().unwrap_or_default().join(","),
                    ),
                ],
            );
            let start_at = crate::pagination::decode_jira_offset_token(
                page_token.as_deref(),
                "jira.sprint.issues",
                query_hash.clone(),
            )?;

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
                .get_sprint_issues_from(id, max_results, requested_fields.clone(), start_at)
                .await
                .with_context(|| {
                    format!(
                        "failed to list issues for Jira sprint `{id}` from {}",
                        client.instance_url()
                    )
                })?;

            let next_start = page
                .next_page_token
                .as_deref()
                .and_then(|s| s.parse::<u64>().ok());
            let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
                crate::pagination::jira_offset_next_token(
                    "jira.sprint.issues",
                    next_start,
                    query_hash,
                )?
            } else {
                None
            };
            let next_command = next_cli_token.as_ref().map(|token| {
                let mut parts = vec![
                    "atla".to_owned(),
                    "jira".to_owned(),
                    "sprint".to_owned(),
                    "issues".to_owned(),
                    id.to_string(),
                ];
                if let Some(fields) = requested_fields.as_ref().filter(|f| !f.is_empty()) {
                    parts.push("--fields".to_owned());
                    parts.push(crate::pagination::quote(&fields.join(",")));
                }
                crate::pagination::next_command(parts, limit, token)
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(
                    &serde_json::json!({"issues": page.issues, "pagination": {"isLast": page.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
                )?,
                OutputFormat::Table => print_issues_with_footer(
                    &page.issues,
                    global,
                    requested_fields.as_deref(),
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_issues(&page.issues, global, requested_fields.as_deref())?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
    }

    Ok(())
}

pub(super) async fn run_sprint_list(
    board_id: u64,
    state: Option<String>,
    limit: u32,
    all: bool,
    page_token: Option<String>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let max_results = if all { u32::MAX } else { limit.clamp(1, 1000) };
    let query_hash = crate::pagination::query_hash(
        "jira.sprint.list",
        &[
            ("board", board_id.to_string()),
            ("state", state.clone().unwrap_or_default()),
        ],
    );
    let start_at = crate::pagination::decode_jira_offset_token(
        page_token.as_deref(),
        "jira.sprint.list",
        query_hash.clone(),
    )?;
    let search = JiraSprintSearch {
        board_id,
        start_at,
        max_results,
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

    let next_start = (!all && matches!(page.is_last, Some(false)))
        .then_some(search.start_at + page.values.len() as u64);
    let next_cli_token =
        crate::pagination::jira_offset_next_token("jira.sprint.list", next_start, query_hash)?;
    let next_command = next_cli_token.as_ref().map(|token| {
        let mut parts = vec![
            "atla".to_owned(),
            "jira".to_owned(),
            "sprint".to_owned(),
            "list".to_owned(),
            "--board".to_owned(),
            search.board_id.to_string(),
        ];
        if let Some(state) = &search.state {
            parts.push("--state".to_owned());
            parts.push(crate::pagination::quote(state));
        }
        crate::pagination::next_command(parts, limit, token)
    });
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => crate::output::print_json(
            &serde_json::json!({"values": page.values, "total": page.total, "pagination": {"isLast": page.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
        )?,
        OutputFormat::Table => print_sprints_with_footer(
            &page,
            global,
            next_command
                .as_deref()
                .map(crate::pagination::next_page_footer),
        )?,
        OutputFormat::Csv | OutputFormat::Keys => {
            print_sprints(&page, global)?;
            if let Some(command) = next_command {
                eprintln!("{}", crate::pagination::next_page_footer(&command));
            }
        }
    }
    Ok(())
}
