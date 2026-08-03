use super::*;

pub(super) async fn run(action: PageCommentAction, global: &Invocation) -> anyhow::Result<()> {
    let PageCommentAction::Add {
        page_id,
        body,
        body_flag,
        body_file,
        parent,
        representation,
        numbered_table_rows,
        mentions,
        resolve_mentions,
        attachments,
        attachment_mode,
    } = action
    else {
        unreachable!("page comment dispatcher passed the wrong variant to add")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        for attachment in &attachments {
            println!(
                "Would PUT {}/wiki/rest/api/content/{}/child/attachment with file `{}` using profile `{profile_name}`",
                profile.confluence_api_base_url(),
                page_id,
                attachment.display()
            );
        }
        println!(
            "Would POST {}/wiki/api/v2/footer-comments using profile `{profile_name}`",
            profile.confluence_api_base_url()
        );
        return Ok(());
    }

    let body = read_body(body.or(body_flag), body_file.as_deref())?;
    let jira_client = if resolve_mentions {
        Some(ctx.jira_client()?)
    } else {
        None
    };
    let markdown_options = markdown_to_adf_options_for_body(
        body.as_deref(),
        representation,
        numbered_table_rows,
        &mentions,
        resolve_mentions,
        jira_client.as_ref(),
    )
    .await?;
    let client = ctx.confluence_client()?;
    let mut uploaded = Vec::new();
    for attachment in &attachments {
        let upload = ConfluenceAttachmentUpload {
            page_id: page_id.clone(),
            file: attachment.clone(),
            comment: None,
            minor_edit: false,
        };
        let page = client
            .upload_page_attachment(&upload)
            .await
            .with_context(|| {
                format!(
                    "failed to upload Confluence attachment `{}` to page `{page_id}` from {}",
                    attachment.display(),
                    client.instance_url()
                )
            })?;
        uploaded.extend(page.results);
    }
    let body = body.map(|body| {
        append_confluence_attachment_references(&body, representation, &uploaded, attachment_mode)
    });
    let (body, representation) = prepare_required_body_with_options(
        body,
        representation,
        markdown_options,
        "missing comment body",
    )?;
    let comment = client
        .add_page_comment(&ConfluenceCommentCreate {
            content_id: page_id,
            parent_comment_id: parent,
            body,
            representation,
        })
        .await
        .with_context(|| {
            format!(
                "failed to add Confluence page comment from {}",
                client.instance_url()
            )
        })?;
    print_comment(&comment, global)?;
    Ok(())
}
