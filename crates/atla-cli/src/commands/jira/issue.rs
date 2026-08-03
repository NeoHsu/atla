use anyhow::Context;
use atla_core::{
    JiraAssigneeTarget, JiraAttachment, JiraIssueAssign, JiraIssueCreate, JiraIssueFieldsQuery,
    JiraIssueList, JiraIssueUpdate, default_issue_fields,
};

use crate::cli::{AttachmentMode, IssueAction, IssueCommand, IssueCommentAction};
use crate::context::AppContext;
use crate::invocation::Invocation;

use super::attachment::run_issue_attachment;
use super::format::{
    can_prompt, issue_fields_for_url, open_web_url, parse_fields, parse_issue_fields,
    parse_label_update, print_comment, print_comments, print_comments_with_footer,
    print_created_issue, print_deleted, print_github_commits, print_github_pull_requests,
    print_issue, print_issue_assign, print_issue_comments_section, print_issue_delete,
    print_issue_fields, print_issue_update, print_issue_with_github, print_issues,
    print_issues_with_footer, print_section_header, print_transition_update, read_optional_text,
    read_required_text, select_transition,
};
use super::link::run_issue_link;
use super::worklog::run_issue_worklog;

mod assign;
mod comment;
mod create;
mod delete;
mod fields;
mod list;
mod transition;
mod update;
mod view;

pub(super) async fn run_issue(command: IssueCommand, global: &Invocation) -> anyhow::Result<()> {
    match command.action {
        action @ IssueAction::List { .. } => list::run(action, global).await?,
        action @ IssueAction::Create { .. } => create::run(action, global).await?,
        action @ IssueAction::Update { .. } => update::run(action, global).await?,
        action @ IssueAction::View { .. } => view::run(action, global).await?,
        action @ IssueAction::Delete { .. } => delete::run(action, global).await?,
        action @ IssueAction::Assign { .. } => assign::run(action, global).await?,
        action @ IssueAction::Transition { .. } => transition::run(action, global).await?,
        action @ IssueAction::Comment { .. } => comment::run(action, global).await?,
        IssueAction::Attachment { action } => run_issue_attachment(action, global).await?,
        IssueAction::Link { action } => run_issue_link(action, global).await?,
        IssueAction::Worklog { action } => run_issue_worklog(action, global).await?,
        action @ IssueAction::Fields { .. } => fields::run(action, global).await?,
    }

    Ok(())
}

fn append_jira_attachment_references(
    body: &str,
    attachments: &[JiraAttachment],
    mode: AttachmentMode,
) -> String {
    if attachments.is_empty() {
        return body.to_owned();
    }

    let mut out = body.trim_end().to_owned();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str("Attachments:\n");
    for attachment in attachments {
        out.push_str("- ");
        out.push_str(&jira_attachment_reference(attachment, mode));
        out.push('\n');
    }
    out.trim_end().to_owned()
}

fn jira_attachment_reference(attachment: &JiraAttachment, mode: AttachmentMode) -> String {
    let filename = attachment
        .filename
        .as_deref()
        .or(attachment.id.as_deref())
        .unwrap_or("attachment");
    let Some(url) = attachment.content.as_deref() else {
        return filename.to_owned();
    };

    if should_embed_attachment(mode, attachment.mime_type.as_deref()) {
        format!("![{filename}]({url})")
    } else {
        format!("[{filename}]({url})")
    }
}

fn should_embed_attachment(mode: AttachmentMode, mime_type: Option<&str>) -> bool {
    !matches!(mode, AttachmentMode::Link)
        && mime_type.is_some_and(|mime_type| mime_type.starts_with("image/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_jira_attachment_links_to_comment_body() {
        let body = append_jira_attachment_references(
            "Please check",
            &[JiraAttachment {
                id: Some("10001".to_owned()),
                filename: Some("error.log".to_owned()),
                mime_type: Some("text/plain".to_owned()),
                size: None,
                author: None,
                created: None,
                content: Some("https://example.atlassian.net/attachment/error.log".to_owned()),
                thumbnail: None,
            }],
            AttachmentMode::Auto,
        );

        assert_eq!(
            body,
            "Please check\n\nAttachments:\n- [error.log](https://example.atlassian.net/attachment/error.log)"
        );
    }

    #[test]
    fn auto_embeds_jira_image_attachments() {
        let body = append_jira_attachment_references(
            "Screenshot attached",
            &[JiraAttachment {
                id: Some("10002".to_owned()),
                filename: Some("screenshot.png".to_owned()),
                mime_type: Some("image/png".to_owned()),
                size: None,
                author: None,
                created: None,
                content: Some("https://example.atlassian.net/attachment/screenshot.png".to_owned()),
                thumbnail: None,
            }],
            AttachmentMode::Auto,
        );

        assert!(body.contains(
            "![screenshot.png](https://example.atlassian.net/attachment/screenshot.png)"
        ));
    }
}
