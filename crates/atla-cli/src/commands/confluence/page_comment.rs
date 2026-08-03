use anyhow::Context;
use atla_core::{
    ConfluenceAttachment, ConfluenceAttachmentUpload, ConfluenceCommentCreate,
    ConfluenceCommentSearch,
};

use crate::cli::{AttachmentMode, BodyRepresentation, OutputFormat, PageCommentAction};
use crate::context::AppContext;
use crate::invocation::Invocation;

use super::format::{
    markdown_to_adf_options_for_body, prepare_required_body_with_options, print_comment,
    print_comments, print_comments_with_footer, print_deleted, read_body,
};

mod add;
mod delete;
mod list;

pub(super) async fn run_page_comment(
    action: PageCommentAction,
    global: &Invocation,
) -> anyhow::Result<()> {
    match action {
        action @ PageCommentAction::List { .. } => list::run(action, global).await?,
        action @ PageCommentAction::Add { .. } => add::run(action, global).await?,
        action @ PageCommentAction::Delete { .. } => delete::run(action, global).await?,
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
