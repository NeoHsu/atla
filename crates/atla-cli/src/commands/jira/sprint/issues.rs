use super::*;

pub(super) async fn run(action: SprintAction, global: &Invocation) -> anyhow::Result<()> {
    let SprintAction::Issues {
        id,
        limit,
        all,
        fields,
        page_token,
    } = action
    else {
        unreachable!("sprint action dispatcher passed the wrong variant to issues")
    };
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
            profile.jira_api_base_url()
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
        crate::pagination::jira_offset_next_token("jira.sprint.issues", next_start, query_hash)?
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
        OutputFormat::Json => global.output().print_json(
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
    Ok(())
}
