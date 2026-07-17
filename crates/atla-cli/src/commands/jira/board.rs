use anyhow::Context;
use atla_core::JiraBoardSearch;

use crate::cli::{BoardAction, BoardCommand, GlobalArgs, OutputFormat};
use crate::context::AppContext;

use super::format::{print_board, print_boards, print_boards_with_footer};

pub(super) async fn run_board(command: BoardCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        BoardAction::List {
            project,
            board_type,
            name,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let max_results = if all { u32::MAX } else { limit.clamp(1, 1000) };
            let project_key_or_id = project.or_else(|| profile.default_project.clone());
            let query_hash = crate::pagination::query_hash(
                "jira.board.list",
                &[
                    ("project", project_key_or_id.clone().unwrap_or_default()),
                    ("type", board_type.clone().unwrap_or_default()),
                    ("name", name.clone().unwrap_or_default()),
                ],
            );
            let start_at = crate::pagination::decode_jira_offset_token(
                page_token.as_deref(),
                "jira.board.list",
                query_hash.clone(),
            )?;
            let search = JiraBoardSearch {
                start_at,
                max_results,
                board_type,
                name,
                project_key_or_id,
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/agile/1.0/board?startAt={}&maxResults={}",
                    profile.jira_api_base_url(),
                    search.start_at,
                    search.max_results
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client.search_boards(&search).await.with_context(|| {
                format!("failed to list Jira boards from {}", client.instance_url())
            })?;

            let next_start = (!all && matches!(page.is_last, Some(false)))
                .then_some(search.start_at + page.values.len() as u64);
            let next_cli_token = crate::pagination::jira_offset_next_token(
                "jira.board.list",
                next_start,
                query_hash,
            )?;
            let next_command = next_cli_token.as_ref().map(|token| {
                let mut parts = vec![
                    "atla".to_owned(),
                    "jira".to_owned(),
                    "board".to_owned(),
                    "list".to_owned(),
                ];
                if let Some(project) = &search.project_key_or_id {
                    parts.push("--project".to_owned());
                    parts.push(crate::pagination::quote(project));
                }
                if let Some(board_type) = &search.board_type {
                    parts.push("--type".to_owned());
                    parts.push(crate::pagination::quote(board_type));
                }
                if let Some(name) = &search.name {
                    parts.push("--name".to_owned());
                    parts.push(crate::pagination::quote(name));
                }
                crate::pagination::next_command(parts, limit, token)
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(
                    &serde_json::json!({"values": page.values, "total": page.total, "pagination": {"isLast": page.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
                )?,
                OutputFormat::Table => print_boards_with_footer(
                    &page,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_boards(&page, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        BoardAction::View { id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!("{}/rest/agile/1.0/board/{id}", profile.jira_api_base_url());
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let board = client.get_board(id).await.with_context(|| {
                format!(
                    "failed to load Jira board `{id}` from {}",
                    client.instance_url()
                )
            })?;
            print_board(&board, global)?;
        }
    }

    Ok(())
}
