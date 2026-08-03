use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::Create {
        project,
        issue_type,
        summary,
        description,
        description_file,
        fields,
        labels,
    } = action
    else {
        unreachable!("issue action dispatcher passed the wrong variant to create")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    if global.dry_run
        && let Some(path) = description_file.as_deref()
    {
        global.output().register_plan_input(path)?;
    }
    let mut parsed_fields = parse_fields(&fields)?;
    if let Some(labels) = labels {
        let values = labels
            .split(',')
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .map(|label| serde_json::Value::String(label.to_owned()))
            .collect::<Vec<_>>();
        parsed_fields.insert("labels".to_owned(), serde_json::Value::Array(values));
    }
    let issue = JiraIssueCreate {
        project_key: project,
        issue_type,
        summary,
        description: read_optional_text(description, description_file.as_deref())?,
        fields: parsed_fields,
    };

    if global.dry_run {
        let url = format!("{}/rest/api/3/issue", profile.jira_api_base_url());
        let body = issue.request_body();
        if global.output == Some(crate::cli::OutputFormat::Json) {
            global.output().print_operation_plan(
                crate::operation::OperationId::JIRA_ISSUE_CREATE,
                profile_name,
                &profile.instance,
                "POST",
                url,
                Some(body),
                Vec::new(),
                Vec::new(),
            )?;
        } else {
            println!("Would POST {url} using profile `{profile_name}`");
            global.output().print_dry_run_body(&body)?;
        }
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let created = client
        .create_issue(&issue)
        .await
        .with_context(|| format!("failed to create Jira issue at {}", client.instance_url()))?;

    print_created_issue(&created, global)?;
    Ok(())
}
