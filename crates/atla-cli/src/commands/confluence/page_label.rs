use anyhow::Context;
use atla_core::ConfluenceLabelSearch;

use crate::cli::{GlobalArgs, OutputFormat, PageLabelAction};
use crate::context::AppContext;

use super::format::{print_deleted, print_labels, print_labels_with_footer};

pub(super) async fn run_page_label(
    action: PageLabelAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        PageLabelAction::List {
            page_id,
            prefix,
            limit,
            all,
            page_token,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let query_hash = crate::pagination::query_hash(
                "confluence.page.label.list",
                &[
                    ("pageId", page_id.clone()),
                    ("prefix", prefix.clone().unwrap_or_default()),
                ],
            );
            let cursor = crate::pagination::decode_confluence_cursor_token(
                page_token.as_deref(),
                "confluence.page.label.list",
                query_hash.clone(),
            )?;
            let search = ConfluenceLabelSearch {
                content_id: page_id,
                prefix,
                limit: if all { u32::MAX } else { limit.clamp(1, 250) },
                cursor,
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

            let next_cli_token = if !all && matches!(labels.is_last, Some(false)) {
                crate::pagination::confluence_cursor_next_token(
                    "confluence.page.label.list",
                    labels.next_cursor.clone(),
                    query_hash,
                )?
            } else {
                None
            };
            let next_command = next_cli_token.as_ref().map(|token| {
                let mut parts = vec![
                    "atla".to_owned(),
                    "confluence".to_owned(),
                    "page".to_owned(),
                    "label".to_owned(),
                    "list".to_owned(),
                    crate::pagination::quote(&search.content_id),
                ];
                if let Some(prefix) = &search.prefix {
                    parts.push("--prefix".to_owned());
                    parts.push(crate::pagination::quote(prefix));
                }
                crate::pagination::next_command(parts, limit, token)
            });
            match global.output.unwrap_or(OutputFormat::Table) {
                OutputFormat::Json => crate::output::print_json(
                    &serde_json::json!({"results": labels.results, "pagination": {"isLast": labels.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
                )?,
                OutputFormat::Table => print_labels_with_footer(
                    &labels,
                    global,
                    next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer),
                )?,
                OutputFormat::Csv | OutputFormat::Keys => {
                    print_labels(&labels, global)?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
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
