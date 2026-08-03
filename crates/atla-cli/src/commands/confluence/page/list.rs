use super::*;

pub(super) async fn run(action: PageAction, global: &Invocation) -> anyhow::Result<()> {
    let PageAction::List {
        space,
        space_id,
        title,
        limit,
        all,
        page_token,
    } = action
    else {
        unreachable!("page action dispatcher passed the wrong variant to list")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let limit = if all { u32::MAX } else { limit.clamp(1, 250) };

    if global.dry_run {
        if let Some(space) = &space {
            let space_url = format!(
                "{}/wiki/api/v2/spaces?keys={space}&limit=1",
                profile.confluence_api_base_url()
            );
            println!("Would GET {space_url} using profile `{profile_name}`");
        }

        let mut url = format!(
            "{}/wiki/api/v2/pages?limit={limit}",
            profile.confluence_api_base_url()
        );
        if let Some(space_id) = &space_id {
            url.push_str(&format!("&space-id={space_id}"));
        } else if space.is_some() {
            url.push_str("&space-id=<resolved-space-id>");
        }
        if let Some(title) = &title {
            url.push_str(&format!("&title={title}"));
        }
        println!("Would GET {url} using profile `{profile_name}`");
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let resolved_space_id = resolve_space_id(&client, space.as_deref(), space_id).await?;
    let cursor = crate::pagination::decode_confluence_page_token(
        page_token.as_deref(),
        resolved_space_id.as_deref(),
        title.as_deref(),
    )?;
    let search = ConfluencePageSearch {
        space_id: resolved_space_id,
        title,
        limit,
        cursor,
    };
    let page = client.list_pages(&search).await.with_context(|| {
        format!(
            "failed to list Confluence pages from {}",
            client.instance_url()
        )
    })?;

    let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
        crate::pagination::confluence_page_next_token(
            page.next_cursor.clone(),
            search.space_id.as_deref(),
            search.title.as_deref(),
        )?
    } else {
        None
    };
    let next_command = next_cli_token.as_ref().map(|token| {
        crate::pagination::confluence_page_next_command(
            search.space_id.as_deref(),
            search.title.as_deref(),
            limit,
            token,
        )
    });

    match global.output.unwrap_or(crate::cli::OutputFormat::Table) {
        crate::cli::OutputFormat::Json => global.output().print_json(&serde_json::json!({
            "results": page.results,
            "pagination": {
                "isLast": page.is_last.unwrap_or(true),
                "nextPageToken": next_cli_token,
                "nextCommand": next_command,
            }
        }))?,
        crate::cli::OutputFormat::Table => {
            let footer = next_command
                .as_deref()
                .map(crate::pagination::next_page_footer);
            print_pages_with_footer(&page.results, global, footer)?;
        }
        crate::cli::OutputFormat::Csv | crate::cli::OutputFormat::Keys => {
            print_pages(&page.results, global)?;
            if let Some(command) = next_command {
                eprintln!("{}", crate::pagination::next_page_footer(&command));
            }
        }
    }
    Ok(())
}
