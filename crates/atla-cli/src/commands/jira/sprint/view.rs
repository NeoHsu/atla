use super::*;

pub(super) async fn run(action: SprintAction, global: &Invocation) -> anyhow::Result<()> {
    let SprintAction::View { id } = action else {
        unreachable!("sprint action dispatcher passed the wrong variant to view")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        let url = format!("{}/rest/agile/1.0/sprint/{id}", profile.jira_api_base_url());
        println!("Would GET {url} using profile `{profile_name}`");
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let sprint = client.get_sprint(id).await.with_context(|| {
        format!(
            "failed to load Jira sprint `{id}` from {}",
            client.instance_url()
        )
    })?;

    print_sprint(&sprint, global)?;
    Ok(())
}
