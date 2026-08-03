use super::*;

pub(super) async fn run(action: SprintAction, global: &Invocation) -> anyhow::Result<()> {
    let SprintAction::Create {
        board,
        name,
        start,
        end,
        goal,
    } = action
    else {
        unreachable!("sprint action dispatcher passed the wrong variant to create")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        println!(
            "Would POST {}/rest/agile/1.0/sprint using profile `{profile_name}`",
            profile.jira_api_base_url()
        );
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let sprint = client
        .create_sprint(&JiraSprintCreate {
            board_id: board,
            name,
            start_date: start,
            end_date: end,
            goal,
        })
        .await
        .with_context(|| {
            format!(
                "failed to create Jira sprint from {}",
                client.instance_url()
            )
        })?;
    print_sprint(&sprint, global)?;
    Ok(())
}
