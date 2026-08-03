use super::*;

pub(super) async fn run(action: SprintAction, global: &Invocation) -> anyhow::Result<()> {
    let SprintAction::Start { id, start, end } = action else {
        unreachable!("sprint action dispatcher passed the wrong variant to start")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        println!(
            "Would PUT {}/rest/agile/1.0/sprint/{} with state active using profile `{profile_name}`",
            profile.jira_api_base_url(),
            id
        );
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let existing = client.get_sprint(id).await.with_context(|| {
        format!(
            "failed to load Jira sprint `{id}` from {}",
            client.instance_url()
        )
    })?;
    if existing.state.as_deref() == Some("active") {
        anyhow::bail!("sprint `{id}` is already active");
    }
    let sprint = client
        .update_sprint(&JiraSprintUpdate {
            id,
            state: Some("active".to_owned()),
            name: None,
            start_date: start,
            end_date: end,
            goal: None,
        })
        .await
        .with_context(|| {
            format!(
                "failed to start Jira sprint `{id}` from {}",
                client.instance_url()
            )
        })?;
    print_sprint(&sprint, global)?;
    Ok(())
}
