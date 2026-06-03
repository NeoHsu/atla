use anyhow::Context;
use atla_core::ConfluenceSearch;

use crate::cli::GlobalArgs;
use crate::context::AppContext;

use super::format::print_search_results;

pub(super) async fn run_search(cql: String, limit: u32, global: &GlobalArgs) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let search = ConfluenceSearch {
        cql,
        limit: limit.clamp(1, 250),
    };

    if global.dry_run {
        println!(
            "Would GET {}/wiki/rest/api/search?limit={} with CQL `{}` using profile `{profile_name}`",
            profile.instance.trim_end_matches('/'),
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

    crate::output::warn_if_truncated(
        matches!(page.is_last, Some(false)),
        page.results.len(),
        "results",
    );

    print_search_results(&page.results, global)
}
