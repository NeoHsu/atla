use anyhow::Context;
use atla_core::{
    JiraAssigneeTarget, JiraBoardPage, JiraBoardSearch, JiraComment, JiraCommentPage,
    JiraCreatedIssue, JiraIssue, JiraIssueAssign, JiraIssueCreate, JiraIssueLabelUpdate,
    JiraIssueList, JiraIssueSearch, JiraIssueUpdate, JiraProject, JiraProjectSearch, JiraSprint,
    JiraSprintPage, JiraSprintSearch, JiraTransition, JiraUser,
};
use dialoguer::Select;
use std::io::{IsTerminal, stdin, stdout};
use std::path::Path;

use crate::cli::{
    BoardAction, BoardCommand, GlobalArgs, IssueAction, IssueCommand, IssueCommentAction,
    JiraCommand, JiraResource, OutputFormat, ProjectAction, ProjectCommand, SprintAction,
    SprintCommand,
};
use crate::context::AppContext;
use crate::output;

pub async fn run(command: JiraCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        JiraResource::Issue(command) => run_issue(command, global).await?,
        JiraResource::Project(command) => run_project(command, global).await?,
        JiraResource::Sprint(command) => run_sprint(command, global).await?,
        JiraResource::Board(command) => run_board(command, global).await?,
        JiraResource::Search { jql, limit } => run_search(jql, limit, global).await?,
    }

    Ok(())
}

async fn run_issue(command: IssueCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        IssueAction::List {
            project,
            status,
            assignee,
            jql,
            limit,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let list = JiraIssueList {
                project_key: project,
                status,
                assignee,
                jql,
                max_results: limit.clamp(1, 5000),
            };
            let search = list
                .to_search(profile.default_project.as_deref())
                .context("failed to build Jira issue list query")?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/search/jql?maxResults={}&fields=summary,status,assignee,issuetype,priority",
                    profile.instance.trim_end_matches('/'),
                    search.max_results
                );
                println!(
                    "Would GET {url} with JQL `{}` using profile `{profile_name}`",
                    search.jql
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client.search_issues(&search).await.with_context(|| {
                format!("failed to list Jira issues from {}", client.instance_url())
            })?;

            print_issues(&page.issues, global)?;
        }
        IssueAction::Create {
            project,
            issue_type,
            summary,
            description,
            description_file,
            fields,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let issue = JiraIssueCreate {
                project_key: project,
                issue_type,
                summary,
                description: read_optional_text(description, description_file.as_deref())?,
                fields: parse_fields(&fields)?,
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue",
                    profile.instance.trim_end_matches('/')
                );
                println!("Would POST {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let created = client.create_issue(&issue).await.with_context(|| {
                format!("failed to create Jira issue at {}", client.instance_url())
            })?;

            print_created_issue(&created, global)?;
        }
        IssueAction::Update {
            key,
            summary,
            description,
            description_file,
            fields,
            labels,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let label_update = parse_label_update(&key, labels.as_deref())?;
            let issue = JiraIssueUpdate {
                issue_id_or_key: key,
                summary,
                description: read_optional_text(description, description_file.as_deref())?,
                fields: parse_fields(&fields)?,
            };
            if issue.is_empty() && label_update.is_empty() {
                anyhow::bail!(
                    "nothing to update; provide --summary, --description, --description-file, --field, or --labels"
                );
            }

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{}",
                    profile.instance.trim_end_matches('/'),
                    issue.issue_id_or_key
                );
                println!("Would PUT {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            if !issue.is_empty() {
                client.update_issue(&issue).await.with_context(|| {
                    format!(
                        "failed to update Jira issue `{}` from {}",
                        issue.issue_id_or_key,
                        client.instance_url()
                    )
                })?;
            }
            if !label_update.is_empty() {
                client
                    .update_issue_labels(&label_update)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to update labels for Jira issue `{}` from {}",
                            label_update.issue_id_or_key,
                            client.instance_url()
                        )
                    })?;
            }

            print_issue_update(&issue.issue_id_or_key, global)?;
        }
        IssueAction::View { key, web } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                if web {
                    println!(
                        "Would open {}/browse/{} using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/'),
                        key
                    );
                    return Ok(());
                }
                let url = format!(
                    "{}/rest/api/3/issue/{}?fields=summary,status,assignee,issuetype,priority",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            if web {
                open_web_url(&format!(
                    "{}/browse/{}",
                    profile.instance.trim_end_matches('/'),
                    key
                ))?;
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let issue = client.get_issue(&key).await.with_context(|| {
                format!(
                    "failed to load Jira issue `{key}` from {}",
                    client.instance_url()
                )
            })?;

            print_issue(&issue, global)?;
        }
        IssueAction::Delete {
            key,
            delete_subtasks,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if !yes && !global.dry_run {
                anyhow::bail!("refusing to delete Jira issue `{key}` without --yes");
            }

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{}?deleteSubtasks={delete_subtasks}",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would DELETE {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .delete_issue(&key, delete_subtasks)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

            print_issue_delete(&key, global)?;
        }
        IssueAction::Assign {
            key,
            to,
            account_id,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let target = if to.eq_ignore_ascii_case("me") {
                JiraAssigneeTarget::Me
            } else if account_id {
                JiraAssigneeTarget::AccountId(to)
            } else {
                JiraAssigneeTarget::Query(to)
            };
            let assign = JiraIssueAssign {
                issue_id_or_key: key,
                target,
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{}/assignee",
                    profile.instance.trim_end_matches('/'),
                    assign.issue_id_or_key
                );
                println!("Would PUT {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let user = client.assign_issue(&assign).await.with_context(|| {
                format!(
                    "failed to assign Jira issue `{}` from {}",
                    assign.issue_id_or_key,
                    client.instance_url()
                )
            })?;

            print_issue_assign(&assign.issue_id_or_key, &user, global)?;
        }
        IssueAction::Transition { key, to } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{}/transitions",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                if let Some(to) = &to {
                    println!(
                        "Would GET {url}, then POST transition `{to}` using profile `{profile_name}`"
                    );
                } else {
                    println!("Would GET {url} using profile `{profile_name}`");
                }
                return Ok(());
            }

            let client = ctx.jira_client()?;
            if let Some(to) = to {
                let transition = client.transition_issue(&key, &to).await.with_context(|| {
                    format!(
                        "failed to transition Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
                print_transition_update(&key, &transition, global)?;
            } else {
                let transitions = client.list_transitions(&key).await.with_context(|| {
                    format!(
                        "failed to list transitions for Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
                if can_prompt(global) && !transitions.is_empty() {
                    let selected = select_transition(&transitions)?;
                    let transition_id = selected
                        .id
                        .as_deref()
                        .or(selected.name.as_deref())
                        .ok_or_else(|| {
                            anyhow::anyhow!("selected transition did not include an id or name")
                        })?;
                    let transition = client
                        .transition_issue(&key, transition_id)
                        .await
                        .with_context(|| {
                            format!(
                                "failed to transition Jira issue `{key}` from {}",
                                client.instance_url()
                            )
                        })?;
                    print_transition_update(&key, &transition, global)?;
                } else {
                    print_transitions(&transitions, global)?;
                }
            }
        }
        IssueAction::Comment { action } => match action {
            IssueCommentAction::Add {
                key,
                body,
                body_file,
            } => {
                let ctx = AppContext::load(global)?;
                let profile_name = ctx.profile_name();
                let profile = ctx.profile();
                let body = read_required_text(body, body_file.as_deref(), "comment body")?;

                if global.dry_run {
                    let url = format!(
                        "{}/rest/api/3/issue/{}/comment",
                        profile.instance.trim_end_matches('/'),
                        key
                    );
                    println!("Would POST {url} using profile `{profile_name}`");
                    return Ok(());
                }

                let client = ctx.jira_client()?;
                let comment = client.add_comment(&key, &body).await.with_context(|| {
                    format!(
                        "failed to add comment to Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

                print_comment(&comment, global)?;
            }
            IssueCommentAction::List { key, limit } => {
                let ctx = AppContext::load(global)?;
                let profile_name = ctx.profile_name();
                let profile = ctx.profile();
                let limit = limit.clamp(1, 1000);

                if global.dry_run {
                    let url = format!(
                        "{}/rest/api/3/issue/{}/comment?startAt=0&maxResults={limit}",
                        profile.instance.trim_end_matches('/'),
                        key
                    );
                    println!("Would GET {url} using profile `{profile_name}`");
                    return Ok(());
                }

                let client = ctx.jira_client()?;
                let page = client.list_comments(&key, limit).await.with_context(|| {
                    format!(
                        "failed to list comments for Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

                print_comments(&page, global)?;
            }
        },
    }

    Ok(())
}

async fn run_search(jql: String, limit: u32, global: &GlobalArgs) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let search = JiraIssueSearch {
        jql,
        max_results: limit.clamp(1, 5000),
    };

    if global.dry_run {
        let url = format!(
            "{}/rest/api/3/search/jql?maxResults={}&fields=summary,status,assignee,issuetype,priority",
            profile.instance.trim_end_matches('/'),
            search.max_results
        );
        println!(
            "Would GET {url} with JQL `{}` using profile `{profile_name}`",
            search.jql
        );
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let page = client.search_issues(&search).await.with_context(|| {
        format!(
            "failed to search Jira issues from {}",
            client.instance_url()
        )
    })?;

    print_issues(&page.issues, global)?;
    Ok(())
}

async fn run_project(command: ProjectCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        ProjectAction::List { query, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = JiraProjectSearch {
                start_at: 0,
                max_results: limit.clamp(1, 100),
                query,
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/project/search?startAt={}&maxResults={}",
                    profile.instance.trim_end_matches('/'),
                    search.start_at,
                    search.max_results
                );
                if let Some(query) = &search.query {
                    println!("Would GET {url} with query `{query}` using profile `{profile_name}`");
                } else {
                    println!("Would GET {url} using profile `{profile_name}`");
                }
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client.search_projects(&search).await.with_context(|| {
                format!(
                    "failed to list Jira projects from {}",
                    client.instance_url()
                )
            })?;

            print_projects(&page.values, page.total, global)?;
        }
        ProjectAction::View { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/project/{}",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let project = client.get_project(&key).await.with_context(|| {
                format!(
                    "failed to load Jira project `{key}` from {}",
                    client.instance_url()
                )
            })?;

            print_project(&project, global)?;
        }
    }

    Ok(())
}

async fn run_board(command: BoardCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        BoardAction::List {
            project,
            board_type,
            name,
            limit,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = JiraBoardSearch {
                start_at: 0,
                max_results: limit.clamp(1, 1000),
                board_type,
                name,
                project_key_or_id: project.or_else(|| profile.default_project.clone()),
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/agile/1.0/board?startAt={}&maxResults={}",
                    profile.instance.trim_end_matches('/'),
                    search.start_at,
                    search.max_results
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client.search_boards(&search).await.with_context(|| {
                format!("failed to list Jira boards from {}", client.instance_url())
            })?;

            print_boards(&page, global)?;
        }
    }

    Ok(())
}

async fn run_sprint(command: SprintCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        SprintAction::List {
            board,
            state,
            limit,
        } => {
            run_sprint_list(board, state, limit, global).await?;
        }
        SprintAction::Active { board, limit } => {
            run_sprint_list(board, Some("active".to_owned()), limit, global).await?;
        }
        SprintAction::View { id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/rest/agile/1.0/sprint/{id}",
                    profile.instance.trim_end_matches('/')
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let sprint = client.get_sprint(id).await.with_context(|| {
                format!(
                    "failed to load Jira sprint `{id}` from {}",
                    client.instance_url()
                )
            })?;

            print_sprint(&sprint, global)?;
        }
    }

    Ok(())
}

async fn run_sprint_list(
    board_id: u64,
    state: Option<String>,
    limit: u32,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let search = JiraSprintSearch {
        board_id,
        start_at: 0,
        max_results: limit.clamp(1, 1000),
        state,
    };

    if global.dry_run {
        let url = format!(
            "{}/rest/agile/1.0/board/{}/sprint?startAt={}&maxResults={}",
            profile.instance.trim_end_matches('/'),
            search.board_id,
            search.start_at,
            search.max_results
        );
        if let Some(state) = &search.state {
            println!("Would GET {url} with state `{state}` using profile `{profile_name}`");
        } else {
            println!("Would GET {url} using profile `{profile_name}`");
        }
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let page = client.list_sprints(&search).await.with_context(|| {
        format!(
            "failed to list Jira sprints for board `{board_id}` from {}",
            client.instance_url()
        )
    })?;

    print_sprints(&page, global)?;
    Ok(())
}

fn print_projects(
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
                    project.archived.unwrap_or(false).to_string(),
                ]
            })
            .collect(),
        total.map(|total| format!("Showing {} of {total} projects.", projects.len())),
    )
}

fn print_issues(issues: &[JiraIssue], global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        issues,
        issues
            .iter()
            .filter_map(|issue| issue.key.clone())
            .collect(),
        &[
            "key", "summary", "status", "assignee", "type", "priority", "id",
        ],
        issues
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
            .collect(),
        None,
    )
}

fn print_issue(issue: &JiraIssue, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(issue),
        OutputFormat::Keys => {
            if let Some(key) = &issue.key {
                println!("{key}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,summary,status,assignee,type,priority,id");
            println!(
                "{},{},{},{},{},{},{}",
                output::csv_cell(issue.key.as_deref().unwrap_or_default()),
                output::csv_cell(issue.summary().unwrap_or_default()),
                output::csv_cell(issue.status_name().unwrap_or_default()),
                output::csv_cell(issue.assignee_display_name().unwrap_or_default()),
                output::csv_cell(issue.issue_type_name().unwrap_or_default()),
                output::csv_cell(issue.priority_name().unwrap_or_default()),
                output::csv_cell(issue.id.as_deref().unwrap_or_default())
            );
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
            Ok(())
        }
    }
}

fn print_created_issue(issue: &JiraCreatedIssue, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_issue_update(key: &str, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_issue_delete(key: &str, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_issue_assign(key: &str, user: &JiraUser, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(&serde_json::json!({
            "key": key,
            "assigned": true,
            "assignee": user
        })),
        OutputFormat::Keys => {
            println!("{key}");
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,assigned,accountId,displayName");
            println!(
                "{},true,{},{}",
                output::csv_cell(key),
                output::csv_cell(user.account_id.as_deref().unwrap_or_default()),
                output::csv_cell(user.display_name.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("Assigned: {key}");
            println!(
                "Assignee: {}",
                user.display_name
                    .as_deref()
                    .or(user.account_id.as_deref())
                    .unwrap_or("-")
            );
            Ok(())
        }
    }
}

fn print_transitions(transitions: &[JiraTransition], global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        transitions,
        transitions
            .iter()
            .filter_map(|transition| transition.id.clone())
            .collect(),
        &["id", "name", "toStatus"],
        transitions
            .iter()
            .map(|transition| {
                vec![
                    transition.id.as_deref().unwrap_or("-").to_owned(),
                    transition.name.as_deref().unwrap_or("-").to_owned(),
                    transition
                        .to_status
                        .as_ref()
                        .and_then(|status| status.name.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                ]
            })
            .collect(),
        None,
    )
}

fn print_transition_update(
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

fn print_comments(page: &JiraCommentPage, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_comment(comment: &JiraComment, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_boards(page: &JiraBoardPage, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_sprints(page: &JiraSprintPage, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_sprint(sprint: &JiraSprint, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_project(project: &JiraProject, global: &GlobalArgs) -> anyhow::Result<()> {
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
                output::csv_cell(&project.archived.unwrap_or(false).to_string()),
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
            println!("Archived: {}", project.archived.unwrap_or(false));
            if let Some(id) = &project.id {
                println!("ID: {id}");
            }
            Ok(())
        }
    }
}

fn read_optional_text(
    value: Option<String>,
    file: Option<&Path>,
) -> anyhow::Result<Option<String>> {
    if let Some(file) = file {
        return std::fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))
            .map(Some);
    }

    Ok(value)
}

fn read_required_text(
    value: Option<String>,
    file: Option<&Path>,
    name: &str,
) -> anyhow::Result<String> {
    read_optional_text(value, file)?.ok_or_else(|| anyhow::anyhow!("missing {name}"))
}

fn parse_label_update(
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

    for operation in labels
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let (action, label) = operation.split_once(':').ok_or_else(|| {
            anyhow::anyhow!("expected --labels add:name,remove:name, got `{operation}`")
        })?;
        if label.is_empty() {
            anyhow::bail!("label cannot be empty in `{operation}`");
        }
        match action {
            "add" => update.add.push(label.to_owned()),
            "remove" => update.remove.push(label.to_owned()),
            _ => anyhow::bail!("unsupported label operation `{action}`; use add or remove"),
        }
    }

    Ok(update)
}

fn parse_fields(fields: &[String]) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
    let mut parsed = serde_json::Map::new();
    for field in fields {
        let (name, value) = field
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected --field name=json, got `{field}`"))?;
        if name.is_empty() {
            anyhow::bail!("field name cannot be empty in `{field}`");
        }
        let value = serde_json::from_str(value)
            .with_context(|| format!("failed to parse JSON value for field `{name}`"))?;
        parsed.insert(name.to_owned(), value);
    }

    Ok(parsed)
}

fn select_transition(transitions: &[JiraTransition]) -> anyhow::Result<&JiraTransition> {
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

fn transition_display(transition: &JiraTransition) -> String {
    let name = transition.name.as_deref().unwrap_or("-");
    let to_status = transition
        .to_status
        .as_ref()
        .and_then(|status| status.name.as_deref());
    if let Some(to_status) = to_status {
        format!("{name} -> {to_status}")
    } else {
        name.to_owned()
    }
}

fn can_prompt(global: &GlobalArgs) -> bool {
    !global.no_input && stdin().is_terminal() && stdout().is_terminal()
}

fn open_web_url(url: &str) -> anyhow::Result<()> {
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
