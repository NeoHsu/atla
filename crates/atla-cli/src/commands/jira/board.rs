use anyhow::Context;
use atla_core::JiraBoardSearch;

use crate::cli::{BoardAction, BoardCommand, GlobalArgs};
use crate::context::AppContext;

use super::format::{print_board, print_boards};

pub(super) async fn run_board(command: BoardCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        BoardAction::List {
            project,
            board_type,
            name,
            limit,
            all,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let max_results = if all { u32::MAX } else { limit.clamp(1, 1000) };
            let search = JiraBoardSearch {
                start_at: 0,
                max_results,
                board_type,
                name,
                project_key_or_id: project.or_else(|| profile.default_project.clone()),
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/agile/1.0/board?startAt={}&maxResults={}",
                    profile.instance.trim_end_matches('/'),
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

            if !all {
                crate::output::warn_if_truncated(
                    matches!(page.is_last, Some(false)),
                    page.values.len(),
                    "boards",
                );
            }

            print_boards(&page, global)?;
        }
        BoardAction::View { id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/rest/agile/1.0/board/{id}",
                    profile.instance.trim_end_matches('/')
                );
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
