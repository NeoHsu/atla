use super::*;

pub(super) async fn run(action: SprintAction, global: &Invocation) -> anyhow::Result<()> {
    let SprintAction::List {
        board,
        state,
        limit,
        all,
        page_token,
    } = action
    else {
        unreachable!("sprint action dispatcher passed the wrong variant to list")
    };
    run_sprint_list(board, state, limit, all, page_token, global).await?;
    Ok(())
}
