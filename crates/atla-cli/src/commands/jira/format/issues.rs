use super::*;

pub(in crate::commands::jira) fn print_projects(
    projects: &[JiraProject],
    total: Option<u64>,
    global: &Invocation,
) -> anyhow::Result<()> {
    print_projects_with_footer(projects, total, global, None)
}

pub(in crate::commands::jira) fn print_projects_with_footer(
    projects: &[JiraProject],
    total: Option<u64>,
    global: &Invocation,
    footer: Option<String>,
) -> anyhow::Result<()> {
    global.output().print_records(
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
        footer.or_else(|| {
            total.map(|total| format!("Showing {} of {total} projects.", projects.len()))
        }),
    )
}

pub(in crate::commands::jira) fn print_issues(
    issues: &[JiraIssue],
    global: &Invocation,
    requested_fields: Option<&[String]>,
) -> anyhow::Result<()> {
    print_issues_with_footer(issues, global, requested_fields, None)
}

pub(in crate::commands::jira) fn print_issues_with_footer(
    issues: &[JiraIssue],
    global: &Invocation,
    requested_fields: Option<&[String]>,
    footer: Option<String>,
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
    global.output().print_records(
        global.output.unwrap_or(OutputFormat::Table),
        issues,
        issues
            .iter()
            .filter_map(|issue| issue.key.clone())
            .collect(),
        &header_refs,
        rows,
        footer,
    )
}

/// Map a column name to a display cell value for an issue.
pub(in crate::commands::jira) fn issue_column_cell(issue: &JiraIssue, col: &str) -> String {
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

pub(in crate::commands::jira) fn print_issue(
    issue: &JiraIssue,
    global: &Invocation,
    requested_fields: Option<&[String]>,
) -> anyhow::Result<()> {
    let extra_fields = display_extra_issue_fields(requested_fields);
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(issue),
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

pub(in crate::commands::jira) fn print_created_issue(
    issue: &JiraCreatedIssue,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(issue),
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

pub(in crate::commands::jira) fn print_issue_update(
    key: &str,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(&serde_json::json!({
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

pub(in crate::commands::jira) fn print_issue_delete(
    key: &str,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(&serde_json::json!({
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

pub(in crate::commands::jira) fn print_issue_assign(
    key: &str,
    user: &JiraUser,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => {
            let is_unassigned = user.account_id.is_none() && user.display_name.is_none();
            global.output().print_json(&serde_json::json!({
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

pub(in crate::commands::jira) fn print_transition_update(
    key: &str,
    transition: &JiraTransition,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(&serde_json::json!({
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

pub(in crate::commands::jira) fn print_comments(
    page: &JiraCommentPage,
    global: &Invocation,
) -> anyhow::Result<()> {
    print_comments_with_footer(page, global, None)
}

pub(in crate::commands::jira) fn print_comments_with_footer(
    page: &JiraCommentPage,
    global: &Invocation,
    footer: Option<String>,
) -> anyhow::Result<()> {
    global.output().print_records(
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
        footer.or_else(|| {
            page.total
                .map(|total| format!("Showing {} of {total} comments.", page.comments.len()))
        }),
    )
}

pub(in crate::commands::jira) fn print_issue_comments_section(
    page: &JiraCommentPage,
    _global: &Invocation,
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

pub(in crate::commands::jira) fn print_comment(
    comment: &JiraComment,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(comment),
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

pub(in crate::commands::jira) fn print_issue_links(
    links: &[JiraIssueLink],
    global: &Invocation,
) -> anyhow::Result<()> {
    global.output().print_records(
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

pub(in crate::commands::jira) fn print_section_header(title: &str) {
    println!("\n── {title} ──");
}

/// Emits a single combined JSON object when `--with-github` is used with `-o json`.
pub(in crate::commands::jira) fn print_issue_with_github(
    issue: &JiraIssue,
    prs: &[JiraGithubPullRequest],
    commits: &[JiraGithubCommit],
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(&serde_json::json!({
            "issue": issue,
            "pull_requests": prs,
            "commits": commits,
        })),
        OutputFormat::Csv => {
            println!("record_type,key,id,title_or_message,status,url");
            println!(
                "issue,{},{},{},{},",
                output::csv_cell(issue.key.as_deref().unwrap_or_default()),
                output::csv_cell(issue.id.as_deref().unwrap_or_default()),
                output::csv_cell(&issue_column_cell(issue, "summary")),
                output::csv_cell(&issue_column_cell(issue, "status")),
            );
            for pr in prs {
                println!(
                    "pull_request,,{},{},{},{}",
                    output::csv_cell(pr.id.as_deref().unwrap_or_default()),
                    output::csv_cell(pr.title.as_deref().unwrap_or_default()),
                    output::csv_cell(pr.status.as_deref().unwrap_or_default()),
                    output::csv_cell(pr.url.as_deref().unwrap_or_default()),
                );
            }
            for commit in commits {
                println!(
                    "commit,,{},{},,{}",
                    output::csv_cell(commit.id.as_deref().unwrap_or_default()),
                    output::csv_cell(commit.message.as_deref().unwrap_or_default()),
                    output::csv_cell(commit.url.as_deref().unwrap_or_default()),
                );
            }
            Ok(())
        }
        OutputFormat::Keys => {
            if let Some(key) = &issue.key {
                println!("{key}");
            }
            for id in prs.iter().filter_map(|pr| pr.id.as_deref()) {
                println!("{id}");
            }
            for id in commits.iter().filter_map(|commit| commit.id.as_deref()) {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Table => unreachable!("table output renders sections in the caller"),
    }
}

pub(in crate::commands::jira) fn print_github_pull_requests(
    prs: &[JiraGithubPullRequest],
    global: &Invocation,
) -> anyhow::Result<()> {
    global.output().print_records(
        global.output.unwrap_or(OutputFormat::Table),
        prs,
        prs.iter().filter_map(|pr| pr.url.clone()).collect(),
        &[
            "status",
            "id",
            "title",
            "author",
            "source",
            "destination",
            "url",
        ],
        prs.iter()
            .map(|pr| {
                vec![
                    pr.status.as_deref().unwrap_or("-").to_owned(),
                    pr.id.as_deref().unwrap_or("-").to_owned(),
                    pr.title.as_deref().unwrap_or("-").to_owned(),
                    pr.author.as_deref().unwrap_or("-").to_owned(),
                    pr.source_branch.as_deref().unwrap_or("-").to_owned(),
                    pr.destination_branch.as_deref().unwrap_or("-").to_owned(),
                    pr.url.as_deref().unwrap_or("-").to_owned(),
                ]
            })
            .collect(),
        None,
    )
}

pub(in crate::commands::jira) fn print_github_commits(
    commits: &[JiraGithubCommit],
    global: &Invocation,
) -> anyhow::Result<()> {
    global.output().print_records(
        global.output.unwrap_or(OutputFormat::Table),
        commits,
        commits.iter().filter_map(|c| c.url.clone()).collect(),
        &["id", "author", "timestamp", "repository", "message", "url"],
        commits
            .iter()
            .map(|c| {
                vec![
                    c.id.as_deref().unwrap_or("-").to_owned(),
                    c.author.as_deref().unwrap_or("-").to_owned(),
                    c.timestamp.as_deref().unwrap_or("-").to_owned(),
                    c.repository.as_deref().unwrap_or("-").to_owned(),
                    c.message
                        .as_deref()
                        .unwrap_or("-")
                        .lines()
                        .next()
                        .unwrap_or("-")
                        .to_owned(),
                    c.url.as_deref().unwrap_or("-").to_owned(),
                ]
            })
            .collect(),
        None,
    )
}
