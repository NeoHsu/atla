use super::*;

pub(super) async fn run(action: PageAction, global: &Invocation) -> anyhow::Result<()> {
    let PageAction::Delete {
        id,
        purge,
        draft,
        yes,
    } = action
    else {
        unreachable!("page action dispatcher passed the wrong variant to delete")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        println!(
            "Would DELETE {}/wiki/api/v2/pages/{} using profile `{profile_name}`",
            profile.confluence_api_base_url(),
            id
        );
        return Ok(());
    }

    if !yes {
        anyhow::bail!("refusing to delete page `{id}` without --yes");
    }

    let client = ctx.confluence_client()?;
    client.delete_page(&id, purge, draft).await.map_err(|err| {
        let msg = err.to_string();
        if purge && msg.contains("404") {
            anyhow::anyhow!(
                "failed to delete Confluence page `{id}` from {}\n\
                    Hint: to purge a page it must first be in the trash; \
                    run without --purge to move it to trash, then retry with --purge",
                client.instance_url()
            )
        } else if msg.contains("404") && !draft {
            anyhow::anyhow!(
                "failed to delete Confluence page `{id}` from {}\n\
                    Hint: if this is a draft page, add the `--draft` flag",
                client.instance_url()
            )
        } else {
            anyhow::anyhow!(
                "failed to delete Confluence page `{id}` from {}: {err}",
                client.instance_url()
            )
        }
    })?;
    print_deleted("page", &id, global)?;
    Ok(())
}
