use super::*;

pub(super) async fn run(action: SprintAction, global: &Invocation) -> anyhow::Result<()> {
    let SprintAction::Active {
        board,
        limit,
        all,
        page_token,
    } = action
    else {
        unreachable!("sprint action dispatcher passed the wrong variant to active")
    };
    run_sprint_list(
        board,
        Some("active".to_owned()),
        limit,
        all,
        page_token,
        global,
    )
    .await?;
    Ok(())
}
