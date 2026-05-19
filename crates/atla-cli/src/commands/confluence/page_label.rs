use anyhow::Context;
use atla_core::ConfluenceLabelSearch;

use crate::cli::{GlobalArgs, PageLabelAction};
use crate::context::AppContext;

use super::format::{print_deleted, print_labels};

pub(super) async fn run_page_label(
    action: PageLabelAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        PageLabelAction::List {
            page_id,
            prefix,
            limit,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceLabelSearch {
                content_id: page_id,
                prefix,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}/labels?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.content_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let labels = client.list_page_labels(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence page labels from {}",
                    client.instance_url()
                )
            })?;
            print_labels(&labels, global)?;
        }
        PageLabelAction::Add { page_id, labels } => {
            if labels.is_empty() {
                anyhow::bail!("provide at least one label");
            }
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/rest/api/content/{}/label using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    page_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let labels = client
                .add_page_labels(&page_id, &labels)
                .await
                .with_context(|| {
                    format!(
                        "failed to add Confluence page labels from {}",
                        client.instance_url()
                    )
                })?;
            print_labels(&labels, global)?;
        }
        PageLabelAction::Remove { page_id, label } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/rest/api/content/{}/label?name={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    page_id,
                    label
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            client
                .remove_page_label(&page_id, &label)
                .await
                .with_context(|| {
                    format!(
                        "failed to remove Confluence page label `{label}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("label", &label, global)?;
        }
    }

    Ok(())
}
