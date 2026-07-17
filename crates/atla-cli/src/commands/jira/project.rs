use anyhow::Context;
use atla_core::JiraProjectSearch;

use crate::cli::{GlobalArgs, OutputFormat, ProjectAction, ProjectCommand};
use crate::context::AppContext;

use super::format::{print_issue_types, print_project, print_projects, print_projects_with_footer};

pub(super) async fn run_project(
    command: ProjectCommand,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match command.action {
        ProjectAction::List {
            query,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let max_results = if all { u32::MAX } else { limit.clamp(1, 100) };
            let query_hash = crate::pagination::query_hash(
                "jira.project.list",
                &[("query", query.clone().unwrap_or_default())],
            );
            let start_at = crate::pagination::decode_jira_offset_token(
                page_token.as_deref(),
                "jira.project.list",
                query_hash.clone(),
            )?;
            let search = JiraProjectSearch {
                start_at,
                max_results,
                query,
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/project/search?startAt={}&maxResults={}",
                    profile.jira_api_base_url(),
                    search.start_at,
                    search.max_results
                );
                if let Some(query) = &search.query {
                    println!("Would GET {url} with query `{query}` using profile `{profile_name}`");
                } else {
                    println!("Would GET {url} using profile `{profile_name}`");
                }
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client.search_projects(&search).await.with_context(|| {
                format!(
                    "failed to list Jira projects from {}",
                    client.instance_url()
                )
            })?;

            let next_start = (!all && matches!(page.is_last, Some(false)))
                .then_some(search.start_at + page.values.len() as u64);
            let next_cli_token = crate::pagination::jira_offset_next_token(
                "jira.project.list",
                next_start,
                query_hash,
            )?;
            let next_command = next_cli_token.as_ref().map(|token| {
                let mut parts = vec![
                    "atla".to_owned(),
                    "jira".to_owned(),
                    "project".to_owned(),
                    "list".to_owned(),
                ];
                if let Some(query) = &search.query {
                    parts.push("--query".to_owned());
                    parts.push(crate::pagination::quote(query));
                }
                crate::pagination::next_command(parts, limit, token)
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(
                    &serde_json::json!({"values": page.values, "total": page.total, "pagination": {"isLast": page.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
                )?,
                OutputFormat::Table => print_projects_with_footer(
                    &page.values,
                    page.total,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_projects(&page.values, page.total, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        ProjectAction::View { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!("{}/rest/api/3/project/{}", profile.jira_api_base_url(), key);
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let project = client.get_project(&key).await.with_context(|| {
                format!(
                    "failed to load Jira project `{key}` from {}",
                    client.instance_url()
                )
            })?;

            print_project(&project, global)?;
        }
        ProjectAction::IssueTypes { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/rest/api/3/project/{} then GET /rest/api/3/issuetype/project using profile `{profile_name}`",
                    profile.jira_api_base_url(),
                    key
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let issue_types = client.list_issue_types(&key).await.with_context(|| {
                format!(
                    "failed to list issue types for Jira project `{key}` from {}",
                    client.instance_url()
                )
            })?;
            print_issue_types(&issue_types, global)?;
        }
    }

    Ok(())
}
