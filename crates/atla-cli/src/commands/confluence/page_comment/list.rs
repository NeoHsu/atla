use super::*;

pub(super) async fn run(action: PageCommentAction, global: &Invocation) -> anyhow::Result<()> {
    let PageCommentAction::List {
        page_id,
        limit,
        all,
        page_token,
    } = action
    else {
        unreachable!("page comment dispatcher passed the wrong variant to list")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let query_hash = crate::pagination::query_hash(
        "confluence.page.comment.list",
        &[("pageId", page_id.clone())],
    );
    let cursor = crate::pagination::decode_confluence_cursor_token(
        page_token.as_deref(),
        "confluence.page.comment.list",
        query_hash.clone(),
    )?;
    let search = ConfluenceCommentSearch {
        content_id: page_id,
        limit: if all { u32::MAX } else { limit.clamp(1, 250) },
        cursor,
    };

    if global.dry_run {
        println!(
            "Would GET {}/wiki/api/v2/pages/{}/footer-comments?limit={} using profile `{profile_name}`",
            profile.confluence_api_base_url(),
            search.content_id,
            search.limit
        );
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let comments = client.list_page_comments(&search).await.with_context(|| {
        format!(
            "failed to list Confluence page comments from {}",
            client.instance_url()
        )
    })?;

    let next_cli_token = if !all && matches!(comments.is_last, Some(false)) {
        crate::pagination::confluence_cursor_next_token(
            "confluence.page.comment.list",
            comments.next_cursor.clone(),
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
                "page".to_owned(),
                "comment".to_owned(),
                "list".to_owned(),
                crate::pagination::quote(&search.content_id),
            ],
            limit,
            token,
        )
    });
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(
            &serde_json::json!({"results": comments.results, "pagination": {"isLast": comments.is_last.unwrap_or(true), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
        )?,
        OutputFormat::Table => print_comments_with_footer(
            &comments,
            global,
            next_command
                .as_deref()
                .map(crate::pagination::next_page_footer),
        )?,
        OutputFormat::Csv | OutputFormat::Keys => {
            print_comments(&comments, global)?;
            if let Some(command) = next_command {
                eprintln!("{}", crate::pagination::next_page_footer(&command));
            }
        }
    }
    Ok(())
}
