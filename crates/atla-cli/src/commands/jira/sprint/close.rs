use super::*;

pub(super) async fn run(action: SprintAction, global: &Invocation) -> anyhow::Result<()> {
    let SprintAction::Close { id } = action else {
        unreachable!("sprint action dispatcher passed the wrong variant to close")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        println!(
            "Would PUT {}/rest/agile/1.0/sprint/{} with state closed using profile `{profile_name}`",
            profile.jira_api_base_url(),
            id
        );
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let sprint = client
        .update_sprint(&JiraSprintUpdate {
            id,
            state: Some("closed".to_owned()),
            name: None,
            start_date: None,
            end_date: None,
            goal: None,
        })
        .await
        .with_context(|| {
            format!(
                "failed to close Jira sprint `{id}` from {}",
                client.instance_url()
            )
        })?;
    print_sprint(&sprint, global)?;
    Ok(())
}
