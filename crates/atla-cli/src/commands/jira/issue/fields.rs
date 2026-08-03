use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::Fields {
        project,
        issue_type,
        required_only,
    } = action
    else {
        unreachable!("issue action dispatcher passed the wrong variant to fields")
    };
    let ctx = AppContext::load(global)?;
    let client = ctx.jira_client()?;

    let issue_types = client
        .list_issue_types(&project)
        .await
        .with_context(|| format!("failed to list issue types for project `{project}`"))?;

    let matched = issue_types
        .iter()
        .find(|t| {
            t.name
                .as_deref()
                .map(|n| n.eq_ignore_ascii_case(&issue_type))
                == Some(true)
                || t.id.as_deref() == Some(issue_type.as_str())
        })
        .ok_or_else(|| {
            anyhow::anyhow!("issue type `{issue_type}` not found in project `{project}`")
        })?;

    let type_id = matched
        .id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("issue type has no id"))?;

    let query = JiraIssueFieldsQuery {
        project_key: project.clone(),
        issue_type_id: type_id.to_owned(),
    };

    let fields = client.get_issue_fields(&query).await.with_context(|| {
        format!("failed to get fields for `{issue_type}` in project `{project}`")
    })?;

    print_issue_fields(&fields, required_only, global)?;
    Ok(())
}
