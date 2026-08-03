use super::*;

pub(super) async fn run(action: PageAction, global: &Invocation) -> anyhow::Result<()> {
    let PageAction::Move { id, parent } = action else {
        unreachable!("page action dispatcher passed the wrong variant to move")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        println!(
            "Would GET {}/wiki/api/v2/pages/{}?body-format=storage using profile `{profile_name}`",
            profile.confluence_api_base_url(),
            id
        );
        println!(
            "Would PUT {}/wiki/api/v2/pages/{} with parentId `{}` using profile `{profile_name}`",
            profile.confluence_api_base_url(),
            id,
            parent
        );
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let page = client.move_page(&id, &parent).await.with_context(|| {
        format!(
            "failed to move Confluence page `{id}` under `{parent}` from {}",
            client.instance_url()
        )
    })?;
    print_page(&page, global)?;
    Ok(())
}
