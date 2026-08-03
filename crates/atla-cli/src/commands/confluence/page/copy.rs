use super::*;

pub(super) async fn run(action: PageAction, global: &Invocation) -> anyhow::Result<()> {
    let PageAction::Copy {
        source_id,
        title,
        space,
        space_id,
        parent,
        root_level,
    } = action
    else {
        unreachable!("page action dispatcher passed the wrong variant to copy")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        if let Some(space) = &space {
            println!(
                "Would GET {}/wiki/api/v2/spaces?keys={space}&limit=1 using profile `{profile_name}`",
                profile.confluence_api_base_url()
            );
        }
        println!(
            "Would GET {}/wiki/api/v2/pages/{}?body-format=storage using profile `{profile_name}`",
            profile.confluence_api_base_url(),
            source_id
        );
        println!(
            "Would POST {}/wiki/api/v2/pages using profile `{profile_name}`",
            profile.confluence_api_base_url()
        );
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let resolved_space_id = resolve_space_id(&client, space.as_deref(), space_id).await?;
    let page = client
        .copy_page(&ConfluencePageCopy {
            source_id: source_id.clone(),
            title,
            space_id: resolved_space_id,
            parent_id: parent,
            root_level,
        })
        .await
        .with_context(|| {
            format!(
                "failed to copy Confluence page `{source_id}` from {}",
                client.instance_url()
            )
        })?;

    print_page(&page, global)?;
    Ok(())
}
