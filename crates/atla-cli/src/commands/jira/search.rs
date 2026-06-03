use anyhow::Context;
use atla_core::JiraIssueSearch;

use crate::cli::{GlobalArgs, OutputFormat};
use crate::context::AppContext;

use super::format::{
    issue_fields_for_url, parse_issue_fields, print_issues, print_issues_with_footer,
};

pub(super) async fn run_search(
    jql: String,
    limit: u32,
    all: bool,
    page_token: Option<String>,
    fields: Option<String>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let requested_fields = parse_issue_fields(fields.as_deref())?;
    let max_results = if all { u32::MAX } else { limit.clamp(1, 5000) };
    let next_page_token = crate::pagination::decode_jira_jql_token(
        page_token.as_deref(),
        &jql,
        requested_fields.as_deref(),
    )?;
    let search = JiraIssueSearch {
        jql,
        max_results,
        fields: requested_fields.clone(),
        next_page_token,
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

    if page.issues.is_empty()
        && matches!(
            global.output.unwrap_or(OutputFormat::Table),
            OutputFormat::Table
        )
    {
        println!("No issues found.");
        return Ok(());
    }

    let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
        crate::pagination::jira_jql_next_token(
            page.next_page_token.clone(),
            &search.jql,
            requested_fields.as_deref(),
        )?
    } else {
        None
    };
    let next_command = next_cli_token.as_ref().map(|token| {
        crate::pagination::jira_search_next_command(
            &search.jql,
            limit,
            requested_fields.as_deref(),
            token,
        )
    });

    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => crate::output::print_json(&serde_json::json!({
            "issues": page.issues,
            "pagination": {
                "isLast": page.is_last.unwrap_or(true),
                "nextPageToken": next_cli_token,
                "nextCommand": next_command,
            }
        }))?,
        OutputFormat::Table => {
            let footer = next_command
                .as_deref()
                .map(crate::pagination::next_page_footer);
            print_issues_with_footer(&page.issues, global, requested_fields.as_deref(), footer)?;
        }
        OutputFormat::Csv | OutputFormat::Keys => {
            print_issues(&page.issues, global, requested_fields.as_deref())?;
            if let Some(command) = next_command {
                eprintln!("{}", crate::pagination::next_page_footer(&command));
            }
        }
    }
    Ok(())
}
