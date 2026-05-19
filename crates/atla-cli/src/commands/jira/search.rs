use anyhow::Context;
use atla_core::JiraIssueSearch;

use crate::cli::{GlobalArgs, OutputFormat};
use crate::context::AppContext;

use super::format::{issue_fields_for_url, parse_issue_fields, print_issues};

pub(super) async fn run_search(
    jql: String,
    limit: u32,
    fields: Option<String>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let requested_fields = parse_issue_fields(fields.as_deref())?;
    let search = JiraIssueSearch {
        jql,
        max_results: limit.clamp(1, 5000),
        fields: requested_fields.clone(),
    };

    if global.dry_run {
        let url = format!(
            "{}/rest/api/3/search/jql?maxResults={}&fields={}",
            profile.instance.trim_end_matches('/'),
            search.max_results,
            issue_fields_for_url(requested_fields.as_deref())
        );
        println!(
            "Would GET {url} with JQL `{}` using profile `{profile_name}`",
            search.jql
        );
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let page = client.search_issues(&search).await.with_context(|| {
        format!(
            "failed to search Jira issues from {}",
            client.instance_url()
        )
    })?;

    if page.issues.is_empty() && matches!(global.output.unwrap_or(OutputFormat::Table), OutputFormat::Table)
    {
        println!("No issues found.");
        return Ok(());
    }

    print_issues(&page.issues, global, requested_fields.as_deref())?;
    Ok(())
}
