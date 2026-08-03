use super::*;

pub(in crate::commands::jira) fn print_worklogs(
    page: &JiraWorklogPage,
    global: &Invocation,
) -> anyhow::Result<()> {
    print_worklogs_with_footer(page, global, None)
}

pub(in crate::commands::jira) fn print_worklogs_with_footer(
    page: &JiraWorklogPage,
    global: &Invocation,
    footer: Option<String>,
) -> anyhow::Result<()> {
    global.output().print_records(
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
        footer.or_else(|| {
            page.total
                .map(|total| format!("Showing {} of {total} worklogs.", page.worklogs.len()))
        }),
    )
}

pub(in crate::commands::jira) fn print_worklog(
    worklog: &JiraWorklog,
    global: &Invocation,
) -> anyhow::Result<()> {
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

pub(in crate::commands::jira) fn print_issue_types(
    types: &[JiraIssueType],
    global: &Invocation,
) -> anyhow::Result<()> {
    global.output().print_records(
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

pub(in crate::commands::jira) fn print_issue_fields(
    fields: &[JiraIssueField],
    required_only: bool,
    global: &Invocation,
) -> anyhow::Result<()> {
    let filtered: Vec<&JiraIssueField> = if required_only {
        fields.iter().filter(|f| f.required).collect()
    } else {
        fields.iter().collect()
    };

    global.output().print_records(
        global.output.unwrap_or(OutputFormat::Table),
        &filtered,
        filtered.iter().map(|f| f.field_id.clone()).collect(),
        &["field_id", "name", "required", "type", "allowed_values"],
        filtered
            .iter()
            .map(|f| {
                let av = if f.allowed_values.is_empty() {
                    "-".to_owned()
                } else {
                    f.allowed_values
                        .iter()
                        .take(5)
                        .map(|v| v.display())
                        .collect::<Vec<_>>()
                        .join(", ")
                        + if f.allowed_values.len() > 5 {
                            ", …"
                        } else {
                            ""
                        }
                };
                vec![
                    f.field_id.clone(),
                    f.name.clone(),
                    f.required.to_string(),
                    f.schema_type.as_deref().unwrap_or("-").to_owned(),
                    av,
                ]
            })
            .collect(),
        None,
    )
}

pub(in crate::commands::jira) fn print_attachment_downloads(
    downloads: &[JiraAttachmentDownload],
    global: &Invocation,
) -> anyhow::Result<()> {
    global.output().print_records(
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

pub(in crate::commands::jira) fn print_attachments(
    attachments: &[JiraAttachment],
    global: &Invocation,
) -> anyhow::Result<()> {
    global.output().print_records(
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

pub(in crate::commands::jira) fn print_boards(
    page: &JiraBoardPage,
    global: &Invocation,
) -> anyhow::Result<()> {
    print_boards_with_footer(page, global, None)
}

pub(in crate::commands::jira) fn print_boards_with_footer(
    page: &JiraBoardPage,
    global: &Invocation,
    footer: Option<String>,
) -> anyhow::Result<()> {
    global.output().print_records(
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
        footer.or_else(|| {
            page.total
                .map(|total| format!("Showing {} of {total} boards.", page.values.len()))
        }),
    )
}

pub(in crate::commands::jira) fn print_board(
    board: &JiraBoard,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(board),
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

pub(in crate::commands::jira) fn print_sprints(
    page: &JiraSprintPage,
    global: &Invocation,
) -> anyhow::Result<()> {
    print_sprints_with_footer(page, global, None)
}

pub(in crate::commands::jira) fn print_sprints_with_footer(
    page: &JiraSprintPage,
    global: &Invocation,
    footer: Option<String>,
) -> anyhow::Result<()> {
    global.output().print_records(
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
        footer.or_else(|| {
            page.total
                .map(|total| format!("Showing {} of {total} sprints.", page.values.len()))
        }),
    )
}

pub(in crate::commands::jira) fn print_sprint(
    sprint: &JiraSprint,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(sprint),
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

pub(in crate::commands::jira) fn print_sprint_issue_move(
    sprint_id: u64,
    issues: &[String],
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(&serde_json::json!({
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

pub(in crate::commands::jira) fn print_deleted(
    kind: &str,
    id: &str,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(&serde_json::json!({
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

pub(in crate::commands::jira) fn print_project(
    project: &JiraProject,
    global: &Invocation,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => global.output().print_json(project),
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
