use anyhow::Context;
use atla_core::JiraIssueLinkCreate;

use crate::cli::{GlobalArgs, IssueLinkAction};
use crate::context::AppContext;

use super::format::{print_deleted, print_issue_links, print_issue_update};

pub(super) async fn run_issue_link(
    action: IssueLinkAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        IssueLinkAction::Add {
            key,
            link_type,
            target,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/api/3/issueLink using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .create_issue_link(&JiraIssueLinkCreate {
                    source_key: key.clone(),
                    target_key: target.clone(),
                    link_type,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to link Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
            print_issue_update(&key, global)?;
        }
        IssueLinkAction::List { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/rest/api/3/issue/{}?fields=issuelinks using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let links = client.list_issue_links(&key).await.with_context(|| {
                format!(
                    "failed to list links for Jira issue `{key}` from {}",
                    client.instance_url()
                )
            })?;
            print_issue_links(&links, global)?;
        }
        IssueLinkAction::Remove { link_id, yes } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if !yes && !global.dry_run {
                anyhow::bail!("refusing to delete Jira issue link `{link_id}` without --yes");
            }

            if global.dry_run {
                println!(
                    "Would DELETE {}/rest/api/3/issueLink/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    link_id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client.delete_issue_link(&link_id).await.with_context(|| {
                format!(
                    "failed to delete Jira issue link `{link_id}` from {}",
                    client.instance_url()
                )
            })?;
            print_deleted("issueLink", &link_id, global)?;
        }
    }

    Ok(())
}
