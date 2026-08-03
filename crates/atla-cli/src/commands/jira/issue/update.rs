use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::Update {
        key,
        summary,
        description,
        description_file,
        fields,
        labels,
    } = action
    else {
        unreachable!("issue action dispatcher passed the wrong variant to update")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let label_update = parse_label_update(&key, labels.as_deref())?;
    let issue = JiraIssueUpdate {
        issue_id_or_key: key,
        summary,
        description: read_optional_text(description, description_file.as_deref())?,
        fields: parse_fields(&fields)?,
    };
    if issue.is_empty() && label_update.is_empty() {
        anyhow::bail!(
            "nothing to update; provide --summary, --description, --description-file, --field, or --labels"
        );
    }

    if global.dry_run {
        let url = format!(
            "{}/rest/api/3/issue/{}",
            profile.jira_api_base_url(),
            issue.issue_id_or_key
        );
        println!("Would PUT {url} using profile `{profile_name}`");
        if !issue.is_empty() {
            global.output().print_dry_run_body(&issue.request_body())?;
        }
        if !label_update.is_empty() {
            global
                .output()
                .print_dry_run_body(&label_update.request_body())?;
        }
        return Ok(());
    }

    let client = ctx.jira_client()?;
    if !issue.is_empty() {
        client.update_issue(&issue).await.with_context(|| {
            format!(
                "failed to update Jira issue `{}` from {}",
                issue.issue_id_or_key,
                client.instance_url()
            )
        })?;
    }
    if !label_update.is_empty() {
        client
            .update_issue_labels(&label_update)
            .await
            .with_context(|| {
                format!(
                    "failed to update labels for Jira issue `{}` from {}",
                    label_update.issue_id_or_key,
                    client.instance_url()
                )
            })?;
    }

    print_issue_update(&issue.issue_id_or_key, global)?;
    Ok(())
}
