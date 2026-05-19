use anyhow::Context;
use atla_core::ConfluenceLabelSearch;

use crate::cli::{BlogLabelAction, GlobalArgs};
use crate::context::AppContext;

use super::format::{print_deleted, print_labels};

pub(super) async fn run_blog_label(
    action: BlogLabelAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        BlogLabelAction::List {
            blog_id,
            prefix,
            limit,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceLabelSearch {
                content_id: blog_id,
                prefix,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/blogposts/{}/labels?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.content_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let labels = client.list_blog_labels(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence blog labels from {}",
                    client.instance_url()
                )
            })?;
            print_labels(&labels, global)?;
        }
        BlogLabelAction::Add { blog_id, labels } => {
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
                    blog_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let labels = client
                .add_blog_labels(&blog_id, &labels)
                .await
                .with_context(|| {
                    format!(
                        "failed to add Confluence blog labels from {}",
                        client.instance_url()
                    )
                })?;
            print_labels(&labels, global)?;
        }
        BlogLabelAction::Remove { blog_id, label } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/rest/api/content/{}/label?name={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    blog_id,
                    label
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            client
                .remove_blog_label(&blog_id, &label)
                .await
                .with_context(|| {
                    format!(
                        "failed to remove Confluence blog label `{label}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("label", &label, global)?;
        }
    }

    Ok(())
}
