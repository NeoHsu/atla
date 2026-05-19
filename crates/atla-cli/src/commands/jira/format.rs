use anyhow::Context;
use atla_core::{
    JiraAttachment, JiraAttachmentDownload, JiraBoard, JiraBoardPage, JiraComment, JiraCommentPage,
    JiraCreatedIssue, JiraIssue, JiraIssueLabelUpdate, JiraIssueLink, JiraIssueType, JiraProject,
    JiraSprint, JiraSprintPage, JiraTransition, JiraUser, JiraWorklog, JiraWorklogPage,
    markdown::adf_to_markdown,
};
use dialoguer::Select;
use std::io::{IsTerminal, stdin, stdout};
use std::path::Path;

use crate::cli::{GlobalArgs, OutputFormat};
use crate::output;

pub(super) fn print_projects(
    projects: &[JiraProject],
    total: Option<u64>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        projects,
        projects
            .iter()
            .filter_map(|project| project.key.clone())
            .collect(),
        &["key", "type", "style", "name", "archived"],
        projects
            .iter()
            .map(|project| {
                vec![
                    project.key.as_deref().unwrap_or("-").to_owned(),
                    project
                        .project_type_key
                        .as_deref()
                        .unwrap_or("-")
                        .to_owned(),
                    project.style.as_deref().unwrap_or("-").to_owned(),
                    project.name.as_deref().unwrap_or("-").to_owned(),
                    project
                        .archived
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "-".to_owned()),
                ]
            })
            .collect(),
        total.map(|total| format!("Showing {} of {total} projects.", projects.len())),
    )
}

pub(super) fn print_issues(
    issues: &[JiraIssue],
    global: &GlobalArgs,
    requested_fields: Option<&[String]>,
) -> anyhow::Result<()> {
    // When explicit --fields are given, show exactly those (key is always included for
    // identification). When no --fields, show the default set.
    let (headers, rows): (Vec<String>, Vec<Vec<String>>) = if let Some(fields) = requested_fields {
        // Ensure "key" is always first if not already requested.
        let mut cols: Vec<String> = Vec::new();
        if !fields.iter().any(|f| f == "key") {
            cols.push("key".to_owned());
        }
        cols.extend(fields.iter().cloned());
        let rows = issues
            .iter()
            .map(|issue| {
                cols.iter()
                    .map(|col| issue_column_cell(issue, col))
                    .collect()
            })
            .collect();
        (cols, rows)
    } else {
        let default_headers: Vec<String> = [
            "key", "summary", "status", "assignee", "type", "priority", "id",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let rows = issues
            .iter()
            .map(|issue| {
                vec![
                    issue.key.as_deref().unwrap_or("-").to_owned(),
                    issue.summary().unwrap_or("-").to_owned(),
                    issue.status_name().unwrap_or("-").to_owned(),
                    issue.assignee_display_name().unwrap_or("-").to_owned(),
                    issue.issue_type_name().unwrap_or("-").to_owned(),
                    issue.priority_name().unwrap_or("-").to_owned(),
                    issue.id.as_deref().unwrap_or("-").to_owned(),
                ]
            })
            .collect();
        (default_headers, rows)
    };

    let header_refs: Vec<&str> = headers.iter().map(String::as_str).collect();
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        issues,
        issues
            .iter()
            .filter_map(|issue| issue.key.clone())
            .collect(),
        &header_refs,
        rows,
        None,
    )
}

/// Map a column name to a display cell value for an issue.
pub(super) fn issue_column_cell(issue: &JiraIssue, col: &str) -> String {
    match col {
        "key" => issue.key.as_deref().unwrap_or("-").to_owned(),
        "summary" => issue.summary().unwrap_or("-").to_owned(),
        "status" => issue.status_name().unwrap_or("-").to_owned(),
        "assignee" => issue.assignee_display_name().unwrap_or("-").to_owned(),
        "type" | "issuetype" => issue.issue_type_name().unwrap_or("-").to_owned(),
        "priority" => issue.priority_name().unwrap_or("-").to_owned(),
        "id" => issue.id.as_deref().unwrap_or("-").to_owned(),
        _ => issue_field_cell(issue, col),
    }
}

pub(super) fn print_issue(
    issue: &JiraIssue,
    global: &GlobalArgs,
    requested_fields: Option<&[String]>,
) -> anyhow::Result<()> {
    let extra_fields = display_extra_issue_fields(requested_fields);
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(issue),
        OutputFormat::Keys => {
            if let Some(key) = &issue.key {
                println!("{key}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            let mut headers = vec![
                "key", "summary", "status", "assignee", "type", "priority", "id",
            ];
            headers.extend(extra_fields.iter().map(String::as_str));
            println!("{}", headers.join(","));
            let mut row = vec![
                output::csv_cell(issue.key.as_deref().unwrap_or_default()),
                output::csv_cell(issue.summary().unwrap_or_default()),
                output::csv_cell(issue.status_name().unwrap_or_default()),
                output::csv_cell(issue.assignee_display_name().unwrap_or_default()),
                output::csv_cell(issue.issue_type_name().unwrap_or_default()),
                output::csv_cell(issue.priority_name().unwrap_or_default()),
                output::csv_cell(issue.id.as_deref().unwrap_or_default()),
            ];
            row.extend(
                extra_fields
                    .iter()
                    .map(|field| output::csv_cell(&issue_field_cell(issue, field))),
            );
            println!("{}", row.join(","));
            Ok(())
        }
        OutputFormat::Table => {
            println!("Key: {}", issue.key.as_deref().unwrap_or("-"));
            println!("Summary: {}", issue.summary().unwrap_or("-"));
            println!("Status: {}", issue.status_name().unwrap_or("-"));
            println!("Assignee: {}", issue.assignee_display_name().unwrap_or("-"));
            println!("Type: {}", issue.issue_type_name().unwrap_or("-"));
            println!("Priority: {}", issue.priority_name().unwrap_or("-"));
            if let Some(id) = &issue.id {
                println!("ID: {id}");
            }
            // When no --fields are requested, show standard metadata fields.
            if requested_fields.is_none() {
                if let Some(reporter) = issue.fields.get("reporter") {
                    let text = value_cell(reporter);
                    if text != "-" {
                        println!("Reporter: {text}");
                    }
                }
                if let Some(arr) = issue
                    .fields
                    .get("labels")
                    .and_then(|v| v.as_array())
                    .filter(|arr| !arr.is_empty())
                {
                    let text = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("Labels: {text}");
                }
                if let Some(created) = issue.fields.get("created").and_then(|v| v.as_str()) {
                    println!("Created: {created}");
                }
                if let Some(updated) = issue.fields.get("updated").and_then(|v| v.as_str()) {
                    println!("Updated: {updated}");
                }
                if let Some(key) = issue
                    .fields
                    .get("parent")
                    .and_then(|p| p.get("key"))
                    .and_then(|v| v.as_str())
                {
                    println!("Parent: {key}");
                }
                if let Some(subtasks) = issue
                    .fields
                    .get("subtasks")
                    .and_then(|v| v.as_array())
                    .filter(|s| !s.is_empty())
                {
                    let keys = subtasks
                        .iter()
                        .filter_map(|s| s.get("key").and_then(|v| v.as_str()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("Subtasks: {keys}");
                }
                if let Some(links) = issue.fields.get("issuelinks").and_then(|v| v.as_array()) {
                    for link in links {
                        let link_type = link
                            .get("type")
                            .and_then(|t| t.get("outward").or_else(|| t.get("inward")))
                            .and_then(|v| v.as_str())
                            .unwrap_or("relates to");
                        let linked_key = link
                            .get("outwardIssue")
                            .or_else(|| link.get("inwardIssue"))
                            .and_then(|i| i.get("key"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("-");
                        let summary = link
                            .get("outwardIssue")
                            .or_else(|| link.get("inwardIssue"))
                            .and_then(|i| i.get("fields"))
                            .and_then(|f| f.get("summary"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if summary.is_empty() {
                            println!("Link: {link_type} {linked_key}");
                        } else {
                            println!("Link: {link_type} {linked_key} — {summary}");
                        }
                    }
                }
                if let Some(desc) = issue.fields.get("description") {
                    let text = adf_to_markdown(desc);
                    if !text.is_empty() {
                        println!("Description:\n{text}");
                    }
                }
            }
            for field in extra_fields {
                println!("{field}: {}", issue_field_cell(issue, &field));
            }
            Ok(())
        }
    }
}

pub(super) fn print_created_issue(
    issue: &JiraCreatedIssue,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(issue),
        OutputFormat::Keys => {
            if let Some(key) = &issue.key {
                println!("{key}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,id,self");
            println!(
                "{},{},{}",
                output::csv_cell(issue.key.as_deref().unwrap_or_default()),
                output::csv_cell(issue.id.as_deref().unwrap_or_default()),
                output::csv_cell(issue.self_url.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("Created: {}", issue.key.as_deref().unwrap_or("-"));
            if let Some(id) = &issue.id {
                println!("ID: {id}");
            }
            Ok(())
        }
    }
}

pub(super) fn print_issue_update(key: &str, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(&serde_json::json!({
            "key": key,
            "updated": true
        })),
        OutputFormat::Keys => {
            println!("{key}");
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,updated");
            println!("{},true", output::csv_cell(key));
            Ok(())
        }
        OutputFormat::Table => {
            println!("Updated: {key}");
            Ok(())
        }
    }
}

pub(super) fn print_issue_delete(key: &str, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(&serde_json::json!({
            "key": key,
            "deleted": true
        })),
        OutputFormat::Keys => {
            println!("{key}");
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,deleted");
            println!("{},true", output::csv_cell(key));
            Ok(())
        }
        OutputFormat::Table => {
            println!("Deleted: {key}");
            Ok(())
        }
    }
}

pub(super) fn print_issue_assign(
    key: &str,
    user: &JiraUser,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => {
            let is_unassigned = user.account_id.is_none() && user.display_name.is_none();
            output::print_json(&serde_json::json!({
                "key": key,
                "assigned": !is_unassigned,
                "assignee": if is_unassigned { serde_json::Value::Null } else { serde_json::to_value(user).unwrap_or(serde_json::Value::Null) }
            }))
        }
        OutputFormat::Keys => {
            println!("{key}");
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,assigned,accountId,displayName");
            println!(
                "{},{},{},{}",
                output::csv_cell(key),
                if user.account_id.is_none() && user.display_name.is_none() {
                    "false"
                } else {
                    "true"
                },
                output::csv_cell(user.account_id.as_deref().unwrap_or_default()),
                output::csv_cell(user.display_name.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            let is_unassigned = user.account_id.is_none() && user.display_name.is_none();
            if is_unassigned {
                println!("Unassigned: {key}");
            } else {
                println!("Assigned: {key}");
                println!(
                    "Assignee: {}",
                    user.display_name
                        .as_deref()
                        .or(user.account_id.as_deref())
                        .unwrap_or("-")
                );
            }
            Ok(())
        }
    }
}

pub(super) fn print_transition_update(
    key: &str,
    transition: &JiraTransition,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(&serde_json::json!({
            "key": key,
            "transitioned": true,
            "transition": transition,
        })),
        OutputFormat::Keys => {
            println!("{key}");
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,transitioned,transitionId,transitionName,toStatus");
            println!(
                "{},true,{},{},{}",
                output::csv_cell(key),
                output::csv_cell(transition.id.as_deref().unwrap_or_default()),
                output::csv_cell(transition.name.as_deref().unwrap_or_default()),
                output::csv_cell(
                    transition
                        .to_status
                        .as_ref()
                        .and_then(|status| status.name.as_deref())
                        .unwrap_or_default()
                )
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("Transitioned: {key}");
            println!("Transition: {}", transition.name.as_deref().unwrap_or("-"));
            if let Some(to_status) = transition
                .to_status
                .as_ref()
                .and_then(|status| status.name.as_deref())
            {
                println!("To status: {to_status}");
            }
            Ok(())
        }
    }
}

pub(super) fn print_comments(page: &JiraCommentPage, global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        page,
        page.comments
            .iter()
            .filter_map(|comment| comment.id.clone())
            .collect(),
        &["id", "author", "created", "updated", "body"],
        page.comments
            .iter()
            .map(|comment| {
                vec![
                    comment.id.as_deref().unwrap_or("-").to_owned(),
                    comment
                        .author_display_name
                        .as_deref()
                        .unwrap_or("-")
                        .to_owned(),
                    comment.created.as_deref().unwrap_or("-").to_owned(),
                    comment.updated.as_deref().unwrap_or("-").to_owned(),
                    comment
                        .body_text
                        .as_deref()
                        .unwrap_or("-")
                        .replace('\n', " "),
                ]
            })
            .collect(),
        page.total
            .map(|total| format!("Showing {} of {total} comments.", page.comments.len())),
    )
}

pub(super) fn print_issue_comments_section(
    page: &JiraCommentPage,
    _global: &GlobalArgs,
) -> anyhow::Result<()> {
    let total = page.total.unwrap_or(page.comments.len() as u32);
    println!();
    if total > page.comments.len() as u32 {
        println!("Comments ({} of {total}):", page.comments.len());
    } else {
        println!("Comments ({total}):");
    }
    for comment in &page.comments {
        let author = comment.author_display_name.as_deref().unwrap_or("-");
        let created = comment.created.as_deref().unwrap_or("-");
        let id = comment.id.as_deref().unwrap_or("-");
        println!("  [{id}] {author} — {created}");
        if let Some(body) = &comment.body_text {
            for line in body.lines().take(5) {
                println!("    {line}");
            }
            if body.lines().count() > 5 {
                println!("    ...");
            }
        }
    }
    Ok(())
}

pub(super) fn print_comment(comment: &JiraComment, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(comment),
        OutputFormat::Keys => {
            if let Some(id) = &comment.id {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,author,created,updated,body");
            println!(
                "{},{},{},{},{}",
                output::csv_cell(comment.id.as_deref().unwrap_or_default()),
                output::csv_cell(comment.author_display_name.as_deref().unwrap_or_default()),
                output::csv_cell(comment.created.as_deref().unwrap_or_default()),
                output::csv_cell(comment.updated.as_deref().unwrap_or_default()),
                output::csv_cell(comment.body_text.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("Comment: {}", comment.id.as_deref().unwrap_or("-"));
            if let Some(author) = &comment.author_display_name {
                println!("Author: {author}");
            }
            if let Some(created) = &comment.created {
                println!("Created: {created}");
            }
            if let Some(body) = &comment.body_text {
                println!("Body: {body}");
            }
            Ok(())
        }
    }
}

pub(super) fn print_issue_links(
    links: &[JiraIssueLink],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        links,
        links.iter().filter_map(|link| link.id.clone()).collect(),
        &["id", "type", "inward", "outward", "summary"],
        links
            .iter()
            .map(|link| {
                let issue = link.outward_issue.as_ref().or(link.inward_issue.as_ref());
                vec![
                    link.id.as_deref().unwrap_or("-").to_owned(),
                    link.link_type.as_deref().unwrap_or("-").to_owned(),
                    link.inward_issue
                        .as_ref()
                        .and_then(|issue| issue.key.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                    link.outward_issue
                        .as_ref()
                        .and_then(|issue| issue.key.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                    issue
                        .and_then(|issue| issue.summary.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                ]
            })
            .collect(),
        None,
    )
}

pub(super) fn print_worklogs(page: &JiraWorklogPage, global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        page,
        page.worklogs
            .iter()
            .filter_map(|worklog| worklog.id.clone())
            .collect(),
        &["id", "author", "time", "seconds", "started", "comment"],
        page.worklogs
            .iter()
            .map(|worklog| {
                vec![
                    worklog.id.as_deref().unwrap_or("-").to_owned(),
                    worklog
                        .author
                        .as_ref()
                        .and_then(|author| author.display_name.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                    worklog.time_spent.as_deref().unwrap_or("-").to_owned(),
                    worklog
                        .time_spent_seconds
                        .map(|seconds| seconds.to_string())
                        .unwrap_or_else(|| "-".to_owned()),
                    worklog.started.as_deref().unwrap_or("-").to_owned(),
                    worklog
                        .comment_text()
                        .unwrap_or_else(|| "-".to_owned())
                        .replace('\n', " "),
                ]
            })
            .collect(),
        page.total
            .map(|total| format!("Showing {} of {total} worklogs.", page.worklogs.len())),
    )
}

pub(super) fn print_worklog(worklog: &JiraWorklog, global: &GlobalArgs) -> anyhow::Result<()> {
    print_worklogs(
        &JiraWorklogPage {
            start_at: 0,
            max_results: 1,
            total: Some(1),
            worklogs: vec![worklog.clone()],
        },
        global,
    )
}

pub(super) fn print_issue_types(
    types: &[JiraIssueType],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        types,
        types
            .iter()
            .filter_map(|issue_type| issue_type.id.clone())
            .collect(),
        &["id", "name", "subtask", "description"],
        types
            .iter()
            .map(|issue_type| {
                vec![
                    issue_type.id.as_deref().unwrap_or("-").to_owned(),
                    issue_type.name.as_deref().unwrap_or("-").to_owned(),
                    issue_type
                        .subtask
                        .map(|subtask| subtask.to_string())
                        .unwrap_or_else(|| "-".to_owned()),
                    issue_type
                        .description
                        .as_deref()
                        .unwrap_or("-")
                        .replace('\n', " "),
                ]
            })
            .collect(),
        None,
    )
}

pub(super) fn print_attachment_downloads(
    downloads: &[JiraAttachmentDownload],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        downloads,
        downloads
            .iter()
            .map(|download| download.path.display().to_string())
            .collect(),
        &["path", "bytes", "id", "filename"],
        downloads
            .iter()
            .map(|download| {
                vec![
                    download.path.display().to_string(),
                    download.bytes.to_string(),
                    download.attachment.id.as_deref().unwrap_or("-").to_owned(),
                    download
                        .attachment
                        .filename
                        .as_deref()
                        .unwrap_or("-")
                        .to_owned(),
                ]
            })
            .collect(),
        Some(format!("Downloaded {} attachment(s).", downloads.len())),
    )
}

pub(super) fn print_attachments(
    attachments: &[JiraAttachment],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        attachments,
        attachments
            .iter()
            .filter_map(|attachment| attachment.id.clone())
            .collect(),
        &["id", "filename", "size"],
        attachments
            .iter()
            .map(|attachment| {
                vec![
                    attachment.id.as_deref().unwrap_or("-").to_owned(),
                    attachment.filename.as_deref().unwrap_or("-").to_owned(),
                    attachment
                        .size
                        .map(|size| size.to_string())
                        .unwrap_or_else(|| "-".to_owned()),
                ]
            })
            .collect(),
        Some(format!("Found {} attachment(s).", attachments.len())),
    )
}

pub(super) fn print_boards(page: &JiraBoardPage, global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        page,
        page.values
            .iter()
            .filter_map(|board| board.id.map(|id| id.to_string()))
            .collect(),
        &["id", "name", "type", "self"],
        page.values
            .iter()
            .map(|board| {
                vec![
                    board.id.map(|id| id.to_string()).unwrap_or("-".to_owned()),
                    board.name.as_deref().unwrap_or("-").to_owned(),
                    board.board_type.as_deref().unwrap_or("-").to_owned(),
                    board.self_url.as_deref().unwrap_or("-").to_owned(),
                ]
            })
            .collect(),
        page.total
            .map(|total| format!("Showing {} of {total} boards.", page.values.len())),
    )
}

pub(super) fn print_board(board: &JiraBoard, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(board),
        OutputFormat::Keys => {
            if let Some(id) = board.id {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,name,type,self");
            println!(
                "{},{},{},{}",
                output::csv_cell(&board.id.map(|id| id.to_string()).unwrap_or_default()),
                output::csv_cell(board.name.as_deref().unwrap_or_default()),
                output::csv_cell(board.board_type.as_deref().unwrap_or_default()),
                output::csv_cell(board.self_url.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!(
                "ID: {}",
                board.id.map(|id| id.to_string()).unwrap_or("-".to_owned())
            );
            println!("Name: {}", board.name.as_deref().unwrap_or("-"));
            println!("Type: {}", board.board_type.as_deref().unwrap_or("-"));
            if let Some(self_url) = &board.self_url {
                println!("Self: {self_url}");
            }
            Ok(())
        }
    }
}

pub(super) fn print_sprints(page: &JiraSprintPage, global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        page,
        page.values
            .iter()
            .filter_map(|sprint| sprint.id.map(|id| id.to_string()))
            .collect(),
        &[
            "id",
            "name",
            "state",
            "originBoardId",
            "startDate",
            "endDate",
            "completeDate",
            "goal",
        ],
        page.values
            .iter()
            .map(|sprint| {
                vec![
                    sprint.id.map(|id| id.to_string()).unwrap_or("-".to_owned()),
                    sprint.name.as_deref().unwrap_or("-").to_owned(),
                    sprint.state.as_deref().unwrap_or("-").to_owned(),
                    sprint
                        .origin_board_id
                        .map(|id| id.to_string())
                        .unwrap_or("-".to_owned()),
                    sprint.start_date.as_deref().unwrap_or("-").to_owned(),
                    sprint.end_date.as_deref().unwrap_or("-").to_owned(),
                    sprint.complete_date.as_deref().unwrap_or("-").to_owned(),
                    sprint.goal.as_deref().unwrap_or("-").to_owned(),
                ]
            })
            .collect(),
        page.total
            .map(|total| format!("Showing {} of {total} sprints.", page.values.len())),
    )
}

pub(super) fn print_sprint(sprint: &JiraSprint, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(sprint),
        OutputFormat::Keys => {
            if let Some(id) = sprint.id {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,name,state,originBoardId,startDate,endDate,completeDate,goal");
            println!(
                "{},{},{},{},{},{},{},{}",
                output::csv_cell(&sprint.id.map(|id| id.to_string()).unwrap_or_default()),
                output::csv_cell(sprint.name.as_deref().unwrap_or_default()),
                output::csv_cell(sprint.state.as_deref().unwrap_or_default()),
                output::csv_cell(
                    &sprint
                        .origin_board_id
                        .map(|id| id.to_string())
                        .unwrap_or_default()
                ),
                output::csv_cell(sprint.start_date.as_deref().unwrap_or_default()),
                output::csv_cell(sprint.end_date.as_deref().unwrap_or_default()),
                output::csv_cell(sprint.complete_date.as_deref().unwrap_or_default()),
                output::csv_cell(sprint.goal.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!(
                "ID: {}",
                sprint.id.map(|id| id.to_string()).unwrap_or("-".to_owned())
            );
            println!("Name: {}", sprint.name.as_deref().unwrap_or("-"));
            println!("State: {}", sprint.state.as_deref().unwrap_or("-"));
            println!(
                "Board: {}",
                sprint
                    .origin_board_id
                    .map(|id| id.to_string())
                    .unwrap_or("-".to_owned())
            );
            if let Some(start_date) = &sprint.start_date {
                println!("Start: {start_date}");
            }
            if let Some(end_date) = &sprint.end_date {
                println!("End: {end_date}");
            }
            if let Some(complete_date) = &sprint.complete_date {
                println!("Complete: {complete_date}");
            }
            if let Some(goal) = &sprint.goal {
                println!("Goal: {goal}");
            }
            Ok(())
        }
    }
}

pub(super) fn print_sprint_issue_move(
    sprint_id: u64,
    issues: &[String],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(&serde_json::json!({
            "sprintId": sprint_id,
            "issues": issues
        })),
        OutputFormat::Keys => {
            for issue in issues {
                println!("{issue}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("sprint_id,issue");
            for issue in issues {
                println!("{},{}", sprint_id, output::csv_cell(issue));
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("Sprint: {sprint_id}");
            println!("Issues: {}", issues.join(", "));
            Ok(())
        }
    }
}

pub(super) fn print_deleted(kind: &str, id: &str, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(&serde_json::json!({
            "deleted": true,
            "kind": kind,
            "id": id
        })),
        OutputFormat::Keys => {
            println!("{id}");
            Ok(())
        }
        OutputFormat::Csv => {
            println!("kind,id,deleted");
            println!("{},{},true", output::csv_cell(kind), output::csv_cell(id));
            Ok(())
        }
        OutputFormat::Table => {
            println!("Deleted {kind} {id}");
            Ok(())
        }
    }
}

pub(super) fn print_project(project: &JiraProject, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(project),
        OutputFormat::Keys => {
            if let Some(key) = &project.key {
                println!("{key}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,name,type,style,archived,id");
            println!(
                "{},{},{},{},{},{}",
                output::csv_cell(project.key.as_deref().unwrap_or_default()),
                output::csv_cell(project.name.as_deref().unwrap_or_default()),
                output::csv_cell(project.project_type_key.as_deref().unwrap_or_default()),
                output::csv_cell(project.style.as_deref().unwrap_or_default()),
                output::csv_cell(
                    &project
                        .archived
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "-".to_owned())
                ),
                output::csv_cell(project.id.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("Key: {}", project.key.as_deref().unwrap_or("-"));
            println!("Name: {}", project.name.as_deref().unwrap_or("-"));
            println!(
                "Type: {}",
                project.project_type_key.as_deref().unwrap_or("-")
            );
            println!("Style: {}", project.style.as_deref().unwrap_or("-"));
            println!(
                "Archived: {}",
                project
                    .archived
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "-".to_owned())
            );
            if let Some(id) = &project.id {
                println!("ID: {id}");
            }
            Ok(())
        }
    }
}

pub(super) fn read_optional_text(
    value: Option<String>,
    file: Option<&Path>,
) -> anyhow::Result<Option<String>> {
    if let Some(file) = file {
        return std::fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))
            .map(Some);
    }

    Ok(value.map(|s| unescape_text(&s)))
}

fn unescape_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek().copied() {
                Some('n') => {
                    chars.next();
                    result.push('\n');
                }
                Some('t') => {
                    chars.next();
                    result.push('\t');
                }
                Some('r') => {
                    chars.next();
                    result.push('\r');
                }
                Some('\\') => {
                    chars.next();
                    result.push('\\');
                }
                _ => {
                    result.push(c);
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub(super) fn read_required_text(
    value: Option<String>,
    file: Option<&Path>,
    name: &str,
) -> anyhow::Result<String> {
    read_optional_text(value, file)?.ok_or_else(|| anyhow::anyhow!("missing {name}"))
}

pub(super) fn parse_label_update(
    issue_id_or_key: &str,
    labels: Option<&str>,
) -> anyhow::Result<JiraIssueLabelUpdate> {
    let mut update = JiraIssueLabelUpdate {
        issue_id_or_key: issue_id_or_key.to_owned(),
        add: Vec::new(),
        remove: Vec::new(),
    };
    let Some(labels) = labels else {
        return Ok(update);
    };

    let parts: Vec<&str> = labels
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();

    // Detect whether the user is using add:/remove: prefix syntax.
    let uses_prefix = parts
        .iter()
        .any(|p| p.starts_with("add:") || p.starts_with("remove:"));

    for part in parts {
        if uses_prefix {
            let (action, label) = part.split_once(':').ok_or_else(|| {
                anyhow::anyhow!(
                    "mixed label syntax: when using add:/remove: prefixes all parts must use them, got `{part}`"
                )
            })?;
            if label.is_empty() {
                anyhow::bail!("label cannot be empty in `{part}`");
            }
            match action {
                "add" => update.add.push(label.to_owned()),
                "remove" => update.remove.push(label.to_owned()),
                _ => anyhow::bail!("unsupported label operation `{action}`; use add or remove"),
            }
        } else {
            // Plain name style: all parts are additions.
            update.add.push(part.to_owned());
        }
    }

    Ok(update)
}

pub(super) fn parse_fields(
    fields: &[String],
) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
    let mut parsed = serde_json::Map::new();
    for field in fields {
        let (name, raw) = field
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected --field name=value, got `{field}`"))?;
        if name.is_empty() {
            anyhow::bail!("field name cannot be empty in `{field}`");
        }
        let value = parse_field_value(name, raw)?;
        parsed.insert(name.to_owned(), value);
    }

    Ok(parsed)
}

/// Parse a field value: try JSON first; if that fails and the value is a plain
/// identifier (no JSON structural chars), auto-wrap it as `{"name": value}`.
pub(super) fn parse_field_value(name: &str, raw: &str) -> anyhow::Result<serde_json::Value> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) {
        return Ok(v);
    }
    // Plain identifier (no braces, brackets, colons, or quotes): wrap as {"name": raw}
    let looks_like_plain = !raw.starts_with('{')
        && !raw.starts_with('[')
        && !raw.starts_with('"')
        && !raw.contains(':');
    if looks_like_plain {
        return Ok(match name {
            "assignee" => serde_json::json!({ "accountId": raw }),
            "parent" => serde_json::json!({ "key": raw }),
            _ => serde_json::json!({ "name": raw }),
        });
    }
    anyhow::bail!(
        "invalid value for field `{name}`: `{raw}`\n  \
         Tip: use JSON (e.g. --field '{name}={{\"name\":\"High\"}}') \
         or a plain name (e.g. --field '{name}=High')"
    )
}

pub(super) fn parse_issue_fields(fields: Option<&str>) -> anyhow::Result<Option<Vec<String>>> {
    let Some(fields) = fields else {
        return Ok(None);
    };
    let parsed = fields
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if parsed.is_empty() {
        anyhow::bail!("--fields must include at least one field");
    }

    Ok(Some(parsed))
}

pub(super) fn issue_fields_for_url(fields: Option<&[String]>) -> String {
    fields
        .filter(|fields| !fields.is_empty())
        .map(|fields| fields.join(","))
        .unwrap_or_else(|| "summary,status,assignee,issuetype,priority".to_owned())
}

pub(super) fn display_extra_issue_fields(fields: Option<&[String]>) -> Vec<String> {
    let Some(fields) = fields else {
        return Vec::new();
    };
    if fields.iter().any(|field| field == "*all") {
        return Vec::new();
    }
    fields
        .iter()
        .filter(|field| {
            !matches!(
                field.as_str(),
                "summary" | "status" | "assignee" | "issuetype" | "priority"
            )
        })
        .cloned()
        .collect()
}

pub(super) fn issue_field_cell(issue: &JiraIssue, field: &str) -> String {
    issue
        .fields
        .get(field)
        .map(value_cell)
        .unwrap_or_else(|| "-".to_owned())
}

pub(super) fn value_cell(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "-".to_owned(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Array(values) => {
            values.iter().map(value_cell).collect::<Vec<_>>().join(", ")
        }
        serde_json::Value::Object(object) => object
            .get("name")
            .or_else(|| object.get("displayName"))
            .or_else(|| object.get("value"))
            .or_else(|| object.get("key"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| serde_json::to_string(value).unwrap_or_else(|_| "-".to_owned())),
    }
}

pub(super) fn select_transition(transitions: &[JiraTransition]) -> anyhow::Result<&JiraTransition> {
    let items = transitions
        .iter()
        .map(transition_display)
        .collect::<Vec<_>>();
    let index = Select::new()
        .with_prompt("Transition")
        .items(&items)
        .default(0)
        .interact()
        .context("failed to read transition selection")?;

    transitions
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("selected transition was out of range"))
}

pub(super) fn transition_display(transition: &JiraTransition) -> String {
    let name = transition.name.as_deref().unwrap_or("-");
    let to_status = transition
        .to_status
        .as_ref()
        .and_then(|status| status.name.as_deref());
    match to_status {
        Some(status) if status != name => format!("{name} → {status}"),
        _ => name.to_owned(),
    }
}

pub(super) fn can_prompt(global: &GlobalArgs) -> bool {
    !global.no_input && stdin().is_terminal() && stdout().is_terminal()
}

pub(super) fn open_web_url(url: &str) -> anyhow::Result<()> {
    let command = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "xdg-open"
    };
    let status = if cfg!(target_os = "windows") {
        std::process::Command::new(command)
            .args(["/C", "start", "", url])
            .status()
    } else {
        std::process::Command::new(command).arg(url).status()
    };

    match status {
        Ok(status) if status.success() => Ok(()),
        _ => {
            println!("{url}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_fields_list() {
        let fields = parse_issue_fields(Some("summary, description,attachment"))
            .expect("issue fields")
            .expect("some fields");

        assert_eq!(
            fields,
            vec![
                "summary".to_owned(),
                "description".to_owned(),
                "attachment".to_owned()
            ]
        );
    }

    #[test]
    fn parses_transition_field_json_values() {
        let fields = parse_fields(&[
            r#"customfield_12345={"value":"Ready"}"#.to_owned(),
            r#"customfield_67890="2026-05-18""#.to_owned(),
        ])
        .expect("transition fields");

        assert_eq!(
            fields["customfield_12345"],
            serde_json::json!({ "value": "Ready" })
        );
        assert_eq!(fields["customfield_67890"], serde_json::json!("2026-05-18"));
    }

    #[test]
    fn requested_all_fields_does_not_expand_table_columns() {
        let fields = vec!["*all".to_owned()];

        assert!(display_extra_issue_fields(Some(&fields)).is_empty());
    }
}
