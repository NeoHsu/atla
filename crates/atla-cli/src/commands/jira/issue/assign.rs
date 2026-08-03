use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::Assign {
        key,
        target,
        account_id,
    } = action
    else {
        unreachable!("issue action dispatcher passed the wrong variant to assign")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let target = if target.unassign {
        JiraAssigneeTarget::Unassign
    } else if let Some(to) = target.to {
        if to.eq_ignore_ascii_case("me") {
            JiraAssigneeTarget::Me
        } else if account_id {
            JiraAssigneeTarget::AccountId(to)
        } else {
            JiraAssigneeTarget::Query(to)
        }
    } else {
        anyhow::bail!("provide --to <user> or --unassign");
    };
    let is_unassign = matches!(&target, JiraAssigneeTarget::Unassign);
    let assign = JiraIssueAssign {
        issue_id_or_key: key,
        target,
    };

    if global.dry_run {
        let url = format!(
            "{}/rest/api/3/issue/{}/assignee",
            profile.jira_api_base_url(),
            assign.issue_id_or_key
        );
        if is_unassign {
            println!("Would PUT {url} (unassign) using profile `{profile_name}`");
        } else {
            println!("Would PUT {url} using profile `{profile_name}`");
        }
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let user = client.assign_issue(&assign).await.with_context(|| {
        format!(
            "failed to assign Jira issue `{}` from {}",
            assign.issue_id_or_key,
            client.instance_url()
        )
    })?;

    print_issue_assign(&assign.issue_id_or_key, &user, global)?;
    Ok(())
}
