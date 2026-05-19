use anyhow::Context;
use atla_core::JiraProjectSearch;

use crate::cli::{GlobalArgs, ProjectAction, ProjectCommand};
use crate::context::AppContext;

use super::format::{print_issue_types, print_project, print_projects};

pub(super) async fn run_project(
    command: ProjectCommand,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match command.action {
        ProjectAction::List { query, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = JiraProjectSearch {
                start_at: 0,
                max_results: limit.clamp(1, 100),
                query,
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/project/search?startAt={}&maxResults={}",
                    profile.instance.trim_end_matches('/'),
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

            print_projects(&page.values, page.total, global)?;
        }
        ProjectAction::View { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/project/{}",
                    profile.instance.trim_end_matches('/'),
                    key
                );
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
                    profile.instance.trim_end_matches('/'),
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
