use super::*;

pub(super) async fn run(action: PageAction, global: &Invocation) -> anyhow::Result<()> {
    let PageAction::Children {
        id,
        depth,
        limit,
        all,
        page_token,
    } = action
    else {
        unreachable!("page action dispatcher passed the wrong variant to children")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let depth = depth.map(|depth| depth.clamp(1, 100));
    let query_hash = crate::pagination::query_hash(
        "confluence.page.children",
        &[
            ("pageId", id.clone()),
            ("depth", depth.map(|d| d.to_string()).unwrap_or_default()),
        ],
    );
    let cursor = crate::pagination::decode_confluence_cursor_token(
        page_token.as_deref(),
        "confluence.page.children",
        query_hash.clone(),
    )?;
    let search = ConfluenceContentTreeSearch {
        page_id: id,
        limit: if all { u32::MAX } else { limit.clamp(1, 250) },
        depth,
        cursor,
    };

    if global.dry_run {
        let endpoint = if let Some(depth) = search.depth {
            format!(
                "{}/wiki/api/v2/pages/{}/descendants?limit={}&depth={depth}",
                profile.confluence_api_base_url(),
                search.page_id,
                search.limit
            )
        } else {
            format!(
                "{}/wiki/api/v2/pages/{}/direct-children?limit={}",
                profile.confluence_api_base_url(),
                search.page_id,
                search.limit
            )
        };
        println!("Would GET {endpoint} using profile `{profile_name}`");
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let children = client.list_page_children(&search).await.with_context(|| {
        format!(
            "failed to list Confluence page children for `{}` from {}",
            search.page_id,
            client.instance_url()
        )
    })?;

    let next_cli_token = if !all && matches!(children.is_last, Some(false)) {
        crate::pagination::confluence_cursor_next_token(
            "confluence.page.children",
            children.next_cursor.clone(),
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
            "children".to_owned(),
            crate::pagination::quote(&search.page_id),
        ];
        if let Some(depth) = search.depth {
            parts.push("--depth".to_owned());
            parts.push(depth.to_string());
        }
        crate::pagination::next_command(parts, limit, token)
    });
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(&serde_json::json!({
            "results": children.results,
            "pagination": {"isLast": children.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}
        }))?,
        OutputFormat::Table => print_content_nodes_with_footer(
            &children.results,
            global,
            next_command
                .as_deref()
                .map(crate::pagination::next_page_footer),
        )?,
        OutputFormat::Csv | OutputFormat::Keys => {
            print_content_nodes(&children.results, global)?;
            if let Some(command) = next_command {
                eprintln!("{}", crate::pagination::next_page_footer(&command));
            }
        }
    }
    Ok(())
}
