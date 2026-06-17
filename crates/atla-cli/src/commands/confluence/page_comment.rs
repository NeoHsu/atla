use anyhow::Context;
use atla_core::{
    ConfluenceAttachment, ConfluenceAttachmentUpload, ConfluenceCommentCreate,
    ConfluenceCommentSearch,
};

use crate::cli::{AttachmentMode, BodyRepresentation, GlobalArgs, OutputFormat, PageCommentAction};
use crate::context::AppContext;

use super::format::{
    markdown_to_adf_options_for_body, prepare_required_body_with_options, print_comment,
    print_comments, print_comments_with_footer, print_deleted, read_body,
};

pub(super) async fn run_page_comment(
    action: PageCommentAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        PageCommentAction::List {
            page_id,
            limit,
            all,
            page_token,
        } => {
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
                    profile.instance.trim_end_matches('/'),
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
                OutputFormat::Json => crate::output::print_json(
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
        }
        PageCommentAction::Add {
            page_id,
            body,
            body_file,
            parent,
            representation,
            numbered_table_rows,
            mentions,
            resolve_mentions,
            attachments,
            attachment_mode,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                for attachment in &attachments {
                    println!(
                        "Would PUT {}/wiki/rest/api/content/{}/child/attachment with file `{}` using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/'),
                        page_id,
                        attachment.display()
                    );
                }
                println!(
                    "Would POST {}/wiki/api/v2/footer-comments using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let body = read_body(body, body_file.as_deref())?;
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
                let page = client.upload_page_attachment(&upload).await.with_context(|| {
                    format!(
                        "failed to upload Confluence attachment `{}` to page `{page_id}` from {}",
                        attachment.display(),
                        client.instance_url()
                    )
                })?;
                uploaded.extend(page.results);
            }
            let body = body.map(|body| {
                append_confluence_attachment_references(
                    &body,
                    representation,
                    &uploaded,
                    attachment_mode,
                )
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
        }
        PageCommentAction::Delete {
            page_id,
            comment_id,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if !yes && !global.dry_run {
                anyhow::bail!("refusing to delete Confluence comment `{comment_id}` without --yes");
            }

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/api/v2/footer-comments/{} for page `{page_id}` using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    comment_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            client
                .delete_page_comment(&comment_id)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete Confluence comment `{comment_id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("comment", &comment_id, global)?;
        }
    }

    Ok(())
}

fn append_confluence_attachment_references(
    body: &str,
    representation: BodyRepresentation,
    attachments: &[ConfluenceAttachment],
    mode: AttachmentMode,
) -> String {
    if attachments.is_empty() {
        return body.to_owned();
    }

    match representation {
        BodyRepresentation::Markdown => append_markdown_attachment_references(
            body,
            attachments,
            mode,
            confluence_attachment_markdown_reference,
        ),
        BodyRepresentation::Wiki => append_wiki_attachment_references(body, attachments, mode),
        BodyRepresentation::Storage => {
            append_storage_attachment_references(body, attachments, mode)
        }
        BodyRepresentation::AtlasDocFormat => {
            append_adf_attachment_references(body, attachments).unwrap_or_else(|| body.to_owned())
        }
    }
}

fn append_markdown_attachment_references<F>(
    body: &str,
    attachments: &[ConfluenceAttachment],
    mode: AttachmentMode,
    reference: F,
) -> String
where
    F: Fn(&ConfluenceAttachment, AttachmentMode) -> String,
{
    let mut out = body.trim_end().to_owned();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str("Attachments:\n");
    for attachment in attachments {
        out.push_str("- ");
        out.push_str(&reference(attachment, mode));
        out.push('\n');
    }
    out.trim_end().to_owned()
}

fn append_wiki_attachment_references(
    body: &str,
    attachments: &[ConfluenceAttachment],
    mode: AttachmentMode,
) -> String {
    let mut out = body.trim_end().to_owned();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str("Attachments:\n");
    for attachment in attachments {
        let title = confluence_attachment_title(attachment);
        if should_embed_confluence_attachment(mode, attachment) {
            out.push_str(&format!("* !{title}!\n"));
        } else if let Some(url) = confluence_attachment_url(attachment) {
            out.push_str(&format!("* [{title}|{url}]\n"));
        } else {
            out.push_str(&format!("* {title}\n"));
        }
    }
    out.trim_end().to_owned()
}

fn append_storage_attachment_references(
    body: &str,
    attachments: &[ConfluenceAttachment],
    mode: AttachmentMode,
) -> String {
    let mut out = body.trim_end().to_owned();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str("<p>Attachments:</p><ul>");
    for attachment in attachments {
        let title = escape_storage_text(&confluence_attachment_title(attachment));
        if should_embed_confluence_attachment(mode, attachment) {
            out.push_str(&format!(
                "<li><ac:image><ri:attachment ri:filename=\"{title}\" /></ac:image></li>"
            ));
        } else if should_view_file_confluence_attachment(mode, attachment) {
            out.push_str(&format!(
                "<li><ac:structured-macro ac:name=\"view-file\"><ac:parameter ac:name=\"name\"><ri:attachment ri:filename=\"{title}\" /></ac:parameter></ac:structured-macro></li>"
            ));
        } else {
            out.push_str(&format!(
                "<li><ac:link><ri:attachment ri:filename=\"{title}\" /></ac:link></li>"
            ));
        }
    }
    out.push_str("</ul>");
    out
}

fn append_adf_attachment_references(
    body: &str,
    attachments: &[ConfluenceAttachment],
) -> Option<String> {
    let mut value = serde_json::from_str::<serde_json::Value>(body).ok()?;
    let content = value.get_mut("content")?.as_array_mut()?;
    content.push(serde_json::json!({
        "type": "paragraph",
        "content": [{"type": "text", "text": "Attachments:"}]
    }));
    content.push(serde_json::json!({
        "type": "bulletList",
        "content": attachments.iter().map(|attachment| {
            let title = confluence_attachment_title(attachment);
            let url = confluence_attachment_url(attachment).unwrap_or_default();
            serde_json::json!({
                "type": "listItem",
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": title,
                        "marks": [{"type": "link", "attrs": {"href": url}}]
                    }]
                }]
            })
        }).collect::<Vec<_>>()
    }));
    serde_json::to_string(&value).ok()
}

fn confluence_attachment_markdown_reference(
    attachment: &ConfluenceAttachment,
    mode: AttachmentMode,
) -> String {
    let title = confluence_attachment_title(attachment);
    let Some(url) = confluence_attachment_url(attachment) else {
        return title;
    };
    if should_embed_confluence_attachment(mode, attachment) {
        format!("![{title}]({url})")
    } else {
        format!("[{title}]({url})")
    }
}

fn confluence_attachment_title(attachment: &ConfluenceAttachment) -> String {
    attachment
        .title
        .as_deref()
        .or(attachment.id.as_deref())
        .unwrap_or("attachment")
        .to_owned()
}

fn confluence_attachment_url(attachment: &ConfluenceAttachment) -> Option<String> {
    attachment
        .webui_link
        .as_deref()
        .or(attachment.download_link.as_deref())
        .map(normalize_confluence_attachment_url)
}

fn normalize_confluence_attachment_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("/wiki/") {
        url.to_owned()
    } else if url.starts_with('/') {
        // Confluence Cloud lives under `/wiki`; root-relative attachment links returned
        // by legacy endpoints otherwise resolve against the Atlassian site root and 404.
        format!("/wiki{url}")
    } else {
        url.to_owned()
    }
}

fn should_embed_confluence_attachment(
    mode: AttachmentMode,
    attachment: &ConfluenceAttachment,
) -> bool {
    !matches!(mode, AttachmentMode::Link)
        && attachment
            .media_type
            .as_deref()
            .is_some_and(|media_type| media_type.starts_with("image/"))
}

fn should_view_file_confluence_attachment(
    mode: AttachmentMode,
    attachment: &ConfluenceAttachment,
) -> bool {
    matches!(mode, AttachmentMode::Embed)
        && attachment
            .media_type
            .as_deref()
            .is_some_and(|media_type| !media_type.starts_with("image/"))
}

fn escape_storage_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attachment(title: &str, media_type: &str) -> ConfluenceAttachment {
        ConfluenceAttachment {
            id: Some("att100".to_owned()),
            status: Some("current".to_owned()),
            title: Some(title.to_owned()),
            page_id: Some("123456".to_owned()),
            blog_post_id: None,
            media_type: Some(media_type.to_owned()),
            media_type_description: None,
            file_id: None,
            file_size: None,
            webui_link: Some(format!("/wiki/download/attachments/123456/{title}")),
            download_link: None,
            version: None,
        }
    }

    #[test]
    fn appends_confluence_markdown_attachment_links() {
        let body = append_confluence_attachment_references(
            "Please check",
            BodyRepresentation::Markdown,
            &[attachment("report.pdf", "application/pdf")],
            AttachmentMode::Auto,
        );

        assert_eq!(
            body,
            "Please check\n\nAttachments:\n- [report.pdf](/wiki/download/attachments/123456/report.pdf)"
        );
    }

    #[test]
    fn appends_confluence_storage_attachment_macros() {
        let body = append_confluence_attachment_references(
            "<p>Please check</p>",
            BodyRepresentation::Storage,
            &[attachment("screenshot.png", "image/png")],
            AttachmentMode::Auto,
        );

        assert!(body.contains("<p>Attachments:</p>"));
        assert!(
            body.contains("<ac:image><ri:attachment ri:filename=\"screenshot.png\" /></ac:image>")
        );
    }

    #[test]
    fn confluence_attachment_links_are_wiki_relative() {
        let mut attachment = attachment("report.pdf", "application/pdf");
        attachment.webui_link = Some(
            "/pages/viewpageattachments.action?pageId=123456&preview=%2F123456%2F100%2Freport.pdf"
                .to_owned(),
        );

        assert_eq!(
            confluence_attachment_url(&attachment).as_deref(),
            Some(
                "/wiki/pages/viewpageattachments.action?pageId=123456&preview=%2F123456%2F100%2Freport.pdf"
            )
        );
    }
}
