use super::*;

pub(super) async fn run(action: PageCommentAction, global: &Invocation) -> anyhow::Result<()> {
    let PageCommentAction::Delete {
        page_id,
        comment_id,
        yes,
    } = action
    else {
        unreachable!("page comment dispatcher passed the wrong variant to delete")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if !yes && !global.dry_run {
        anyhow::bail!("refusing to delete Confluence comment `{comment_id}` without --yes");
    }

    if global.dry_run {
        println!(
            "Would DELETE {}/wiki/api/v2/footer-comments/{} for page `{page_id}` using profile `{profile_name}`",
            profile.confluence_api_base_url(),
            comment_id
        );
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    client
        .delete_page_comment(&comment_id)
        .await
        .with_context(|| {
            format!(
                "failed to delete Confluence comment `{comment_id}` from {}",
                client.instance_url()
            )
        })?;
    print_deleted("comment", &comment_id, global)?;
    Ok(())
}
