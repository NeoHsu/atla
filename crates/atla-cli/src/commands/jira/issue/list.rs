use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::List {
        project,
        status,
        issue_type,
        assignee,
        jql,
        limit,
        all,
        page_token,
        fields,
    } = action
    else {
        unreachable!("issue action dispatcher passed the wrong variant to list")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let requested_fields = parse_issue_fields(fields.as_deref())?;
    let max_results = if all { u32::MAX } else { limit.clamp(1, 5000) };
    let list = JiraIssueList {
        project_key: project,
        status,
        issue_type,
        assignee,
        jql,
        max_results,
        fields: requested_fields.clone(),
    };
    let mut search = list
        .to_search(profile.default_project.as_deref())
        .context("failed to build Jira issue list query")?;
    search.next_page_token = crate::pagination::decode_jira_jql_token(
        page_token.as_deref(),
        &search.jql,
        requested_fields.as_deref(),
    )?;

    if global.dry_run {
        let url = format!(
            "{}/rest/api/3/search/jql?maxResults={}&fields={}",
            profile.jira_api_base_url(),
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
    let page = client
        .search_issues(&search)
        .await
        .with_context(|| format!("failed to list Jira issues from {}", client.instance_url()))?;

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

    match global.output.unwrap_or(crate::cli::OutputFormat::Table) {
        crate::cli::OutputFormat::Json => global.output().print_json(&serde_json::json!({
            "issues": page.issues,
            "pagination": {
                "isLast": page.is_last.unwrap_or(true),
                "nextPageToken": next_cli_token,
                "nextCommand": next_command,
            }
        }))?,
        crate::cli::OutputFormat::Table => {
            let footer = next_command
                .as_deref()
                .map(crate::pagination::next_page_footer);
            print_issues_with_footer(&page.issues, global, requested_fields.as_deref(), footer)?;
        }
        crate::cli::OutputFormat::Csv | crate::cli::OutputFormat::Keys => {
            print_issues(&page.issues, global, requested_fields.as_deref())?;
            if let Some(command) = next_command {
                eprintln!("{}", crate::pagination::next_page_footer(&command));
            }
        }
    }
    Ok(())
}
