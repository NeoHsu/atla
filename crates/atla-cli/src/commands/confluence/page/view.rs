use super::*;

pub(super) async fn run(action: PageAction, global: &Invocation) -> anyhow::Result<()> {
    let PageAction::View {
        id,
        web,
        format,
        metadata_only: _metadata_only,
        fields,
        max_chars,
        preserve_table_options,
        with_attachments,
    } = action
    else {
        unreachable!("page action dispatcher passed the wrong variant to view")
    };
    if preserve_table_options && !matches!(format, Some(ContentViewFormat::Markdown)) {
        anyhow::bail!("--preserve-table-options requires --format markdown");
    }
    let markdown_options = AdfToMarkdownOptions {
        table_numbered_rows_directives: preserve_table_options,
    };
    let fields = parse_view_fields(fields.as_deref(), global)?;
    let max_chars = max_chars.map(|value| value as usize);
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        let url = format!(
            "{}/wiki/api/v2/pages/{}",
            profile.confluence_api_base_url(),
            id
        );
        println!("Would GET {url} using profile `{profile_name}`");
        if with_attachments {
            let att_url = format!(
                "{}/wiki/api/v2/pages/{}/attachments",
                profile.confluence_api_base_url(),
                id
            );
            println!("Would GET {att_url} using profile `{profile_name}`");
        }
        return Ok(());
    }

    if web {
        open_web_url(&format!(
            "{}/wiki/pages/viewpage.action?pageId={}",
            profile.instance.trim_end_matches('/'),
            id
        ))?;
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let page = client
        .get_page_with_body_format(&id, format.and_then(view_format_body_representation))
        .await
        .with_context(|| {
            format!(
                "failed to load Confluence page `{id}` from {}",
                client.instance_url()
            )
        })?;

    let attachments = if with_attachments {
        let search = ConfluenceAttachmentSearch {
            page_id: id.clone(),
            filename: None,
            limit: 250,
            cursor: None,
        };
        Some(
            client
                .list_page_attachments(&search)
                .await
                .with_context(|| format!("failed to list attachments for page `{id}`"))?
                .results,
        )
    } else {
        None
    };
    if let Some(format) = format {
        print_page_body_view(
            &page,
            format,
            attachments.as_deref(),
            global,
            markdown_options,
            max_chars,
            fields.as_deref(),
        )?;
    } else {
        print_page_metadata_view(
            &page,
            &id,
            profile_name,
            attachments.as_deref(),
            fields.as_deref(),
            global,
        )?;
    }
    Ok(())
}
