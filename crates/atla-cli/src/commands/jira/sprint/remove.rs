use super::*;

pub(super) async fn run(action: SprintAction, global: &Invocation) -> anyhow::Result<()> {
    let SprintAction::Remove { id, issues } = action else {
        unreachable!("sprint action dispatcher passed the wrong variant to remove")
    };
    if issues.is_empty() {
        anyhow::bail!("provide at least one issue with --issues");
    }
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        println!(
            "Would POST {}/rest/agile/1.0/backlog/issue for sprint `{id}` using profile `{profile_name}`",
            profile.jira_api_base_url()
        );
        return Ok(());
    }

    let client = ctx.jira_client()?;
    client
        .move_issues_to_backlog(&issues)
        .await
        .with_context(|| {
            format!(
                "failed to remove issues from Jira sprint `{id}` via backlog move from {}",
                client.instance_url()
            )
        })?;
    print_sprint_issue_move(id, &issues, global)?;
    Ok(())
}
