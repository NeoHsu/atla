use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::Delete {
        key,
        delete_subtasks,
        yes,
    } = action
    else {
        unreachable!("issue action dispatcher passed the wrong variant to delete")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if !yes && !global.dry_run {
        anyhow::bail!("refusing to delete Jira issue `{key}` without --yes");
    }

    if global.dry_run {
        let url = format!(
            "{}/rest/api/3/issue/{}?deleteSubtasks={delete_subtasks}",
            profile.jira_api_base_url(),
            key
        );
        println!("Would DELETE {url} using profile `{profile_name}`");
        return Ok(());
    }

    let client = ctx.jira_client()?;
    client
        .delete_issue(&key, delete_subtasks)
        .await
        .with_context(|| {
            format!(
                "failed to delete Jira issue `{key}` from {}",
                client.instance_url()
            )
        })?;

    print_issue_delete(&key, global)?;
    Ok(())
}
