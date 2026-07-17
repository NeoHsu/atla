use anyhow::Context;
use atla_core::ConfluenceSearch;

use crate::cli::GlobalArgs;
use crate::context::AppContext;

use super::format::print_search_results;

pub(super) async fn run_search(
    cql: String,
    limit: u32,
    all: bool,
    page_token: Option<String>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let query_hash = crate::pagination::query_hash("confluence.search", &[("cql", cql.clone())]);
    let start = crate::pagination::decode_confluence_offset_token(
        page_token.as_deref(),
        "confluence.search",
        query_hash.clone(),
    )?;
    let search = ConfluenceSearch {
        cql,
        limit: if all { u32::MAX } else { limit.clamp(1, 250) },
        start,
    };

    if global.dry_run {
        println!(
            "Would GET {}/wiki/rest/api/search?limit={} with CQL `{}` using profile `{profile_name}`",
            profile.confluence_api_base_url(),
            search.limit,
            search.cql
        );
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let page = client.search(&search).await.with_context(|| {
        format!(
            "failed to search Confluence content from {}",
            client.instance_url()
        )
    })?;

    let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
        crate::pagination::confluence_offset_next_token(
            "confluence.search",
            page.next_start,
            query_hash,
        )?
    } else {
        None
    };
    let next_command = next_cli_token.as_ref().map(|token| {
        crate::pagination::next_command(
            vec![
                "atla".to_owned(),
                "confluence".to_owned(),
                "search".to_owned(),
                crate::pagination::quote(&search.cql),
            ],
            limit,
            token,
        )
    });

    match global.output.unwrap_or(crate::cli::OutputFormat::Table) {
        crate::cli::OutputFormat::Json => crate::output::print_json(&serde_json::json!({
            "results": page.results,
            "pagination": {
                "isLast": page.is_last.unwrap_or(true),
                "nextPageToken": next_cli_token,
                "nextCommand": next_command,
            }
        })),
        crate::cli::OutputFormat::Table => {
            let footer = next_command
                .as_deref()
                .map(crate::pagination::next_page_footer);
            print_search_results(&page.results, global, footer)
        }
        crate::cli::OutputFormat::Csv | crate::cli::OutputFormat::Keys => {
            print_search_results(&page.results, global, None)?;
            if let Some(command) = next_command {
                eprintln!("{}", crate::pagination::next_page_footer(&command));
            }
            Ok(())
        }
    }
}
