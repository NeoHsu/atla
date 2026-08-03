use super::*;

pub(super) async fn run(action: BlogAction, global: &Invocation) -> anyhow::Result<()> {
    let BlogAction::List {
        space,
        space_id,
        title,
        limit,
        all,
        page_token,
    } = action
    else {
        unreachable!("blog action dispatcher passed the wrong variant to list")
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
            "{}/wiki/api/v2/blogposts?limit={limit}",
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
    let query_hash = crate::pagination::query_hash(
        "confluence.blog.list",
        &[
            ("spaceId", resolved_space_id.clone().unwrap_or_default()),
            ("title", title.clone().unwrap_or_default()),
        ],
    );
    let cursor = crate::pagination::decode_confluence_cursor_token(
        page_token.as_deref(),
        "confluence.blog.list",
        query_hash.clone(),
    )?;
    let search = ConfluenceBlogPostSearch {
        space_id: resolved_space_id,
        title,
        limit,
        cursor,
    };
    let page = client.list_blog_posts(&search).await.with_context(|| {
        format!(
            "failed to list Confluence blog posts from {}",
            client.instance_url()
        )
    })?;

    let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
        crate::pagination::confluence_cursor_next_token(
            "confluence.blog.list",
            page.next_cursor.clone(),
            query_hash,
        )?
    } else {
        None
    };
    let next_command = next_cli_token.as_ref().map(|token| {
        let mut parts = vec![
            "atla".to_owned(),
            "confluence".to_owned(),
            "blog".to_owned(),
            "list".to_owned(),
        ];
        if let Some(space_id) = &search.space_id {
            parts.push("--space-id".to_owned());
            parts.push(crate::pagination::quote(space_id));
        }
        if let Some(title) = &search.title {
            parts.push("--title".to_owned());
            parts.push(crate::pagination::quote(title));
        }
        crate::pagination::next_command(parts, limit, token)
    });
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(&serde_json::json!({
            "results": page.results,
            "pagination": { "isLast": page.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command }
        }))?,
        OutputFormat::Table => print_blog_posts_with_footer(
            &page.results,
            global,
            next_command
                .as_deref()
                .map(crate::pagination::next_page_footer),
        )?,
        OutputFormat::Csv | OutputFormat::Keys => {
            print_blog_posts(&page.results, global)?;
            if let Some(command) = next_command {
                eprintln!("{}", crate::pagination::next_page_footer(&command));
            }
        }
    }
    Ok(())
}
