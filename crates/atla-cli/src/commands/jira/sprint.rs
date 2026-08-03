use anyhow::Context;
use atla_core::{JiraSprintCreate, JiraSprintSearch, JiraSprintUpdate};

use crate::cli::{OutputFormat, SprintAction, SprintCommand};
use crate::context::AppContext;
use crate::invocation::Invocation;

use super::format::{
    parse_issue_fields, print_issues, print_issues_with_footer, print_sprint,
    print_sprint_issue_move, print_sprints, print_sprints_with_footer,
};

mod active;
mod add;
mod close;
mod create;
mod issues;
mod list;
mod remove;
mod start;
mod view;

pub(super) async fn run_sprint(command: SprintCommand, global: &Invocation) -> anyhow::Result<()> {
    match command.action {
        action @ SprintAction::List { .. } => list::run(action, global).await?,
        action @ SprintAction::Active { .. } => active::run(action, global).await?,
        action @ SprintAction::View { .. } => view::run(action, global).await?,
        action @ SprintAction::Create { .. } => create::run(action, global).await?,
        action @ SprintAction::Start { .. } => start::run(action, global).await?,
        action @ SprintAction::Close { .. } => close::run(action, global).await?,
        action @ SprintAction::Add { .. } => add::run(action, global).await?,
        action @ SprintAction::Remove { .. } => remove::run(action, global).await?,
        action @ SprintAction::Issues { .. } => issues::run(action, global).await?,
    }

    Ok(())
}

pub(super) async fn run_sprint_list(
    board_id: u64,
    state: Option<String>,
    limit: u32,
    all: bool,
    page_token: Option<String>,
    global: &Invocation,
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
            profile.jira_api_base_url(),
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
        OutputFormat::Json => global.output().print_json(
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
