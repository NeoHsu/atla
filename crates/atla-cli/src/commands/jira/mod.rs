use anyhow::Context;
use atla_core::auth::{CredentialStore, KeyringCredentialStore};
use atla_core::{
    AtlaConfig, AtlassianClient, ConfigStore, JiraAssigneeTarget, JiraBoardPage, JiraBoardSearch,
    JiraClient, JiraComment, JiraCommentPage, JiraCreatedIssue, JiraIssue, JiraIssueAssign,
    JiraIssueCreate, JiraIssueList, JiraIssueSearch, JiraIssueUpdate, JiraProject,
    JiraProjectSearch, JiraSprint, JiraSprintPage, JiraSprintSearch, JiraTransition, JiraUser,
    Profile,
};
use std::path::Path;

use crate::cli::{
    BoardAction, BoardCommand, GlobalArgs, IssueAction, IssueCommand, IssueCommentAction,
    JiraCommand, JiraResource, OutputFormat, ProjectAction, ProjectCommand, SprintAction,
    SprintCommand,
};
use crate::config;
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
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
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

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
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

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
        } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
            let issue = JiraIssueUpdate {
                issue_id_or_key: key,
                summary,
                description: read_optional_text(description, description_file.as_deref())?,
                fields: parse_fields(&fields)?,
            };
            if issue.is_empty() {
                anyhow::bail!(
                    "nothing to update; provide --summary, --description, --description-file, or --field"
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

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
            client.update_issue(&issue).await.with_context(|| {
                format!(
                    "failed to update Jira issue `{}` from {}",
                    issue.issue_id_or_key,
                    client.instance_url()
                )
            })?;

            print_issue_update(&issue.issue_id_or_key, global)?;
        }
        IssueAction::View { key } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{}?fields=summary,status,assignee,issuetype,priority",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

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

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
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

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

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

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
                print_transitions(&transitions, global)?;
            }
        }
        IssueAction::Comment { action } => match action {
            IssueCommentAction::Add {
                key,
                body,
                body_file,
            } => {
                let store =
                    ConfigStore::default_store().context("failed to find config location")?;
                let atla_config = store.load().context("failed to load config")?;
                let (profile_name, profile) = active_profile(&atla_config, global)?;
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

                let token = token_for_profile(profile_name, profile)?;
                let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
                let comment = client.add_comment(&key, &body).await.with_context(|| {
                    format!(
                        "failed to add comment to Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

                print_comment(&comment, global)?;
            }
            IssueCommentAction::List { key, limit } => {
                let store =
                    ConfigStore::default_store().context("failed to find config location")?;
                let atla_config = store.load().context("failed to load config")?;
                let (profile_name, profile) = active_profile(&atla_config, global)?;
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

                let token = token_for_profile(profile_name, profile)?;
                let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let atla_config = store.load().context("failed to load config")?;
    let (profile_name, profile) = active_profile(&atla_config, global)?;
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

    let token = token_for_profile(profile_name, profile)?;
    let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
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

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
            let page = client.search_projects(&search).await.with_context(|| {
                format!(
                    "failed to list Jira projects from {}",
                    client.instance_url()
                )
            })?;

            print_projects(&page.values, page.total, global)?;
        }
        ProjectAction::View { key } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/project/{}",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
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

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/agile/1.0/sprint/{id}",
                    profile.instance.trim_end_matches('/')
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
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
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let atla_config = store.load().context("failed to load config")?;
    let (profile_name, profile) = active_profile(&atla_config, global)?;
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

    let token = token_for_profile(profile_name, profile)?;
    let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
    let page = client.list_sprints(&search).await.with_context(|| {
        format!(
            "failed to list Jira sprints for board `{board_id}` from {}",
            client.instance_url()
        )
    })?;

    print_sprints(&page, global)?;
    Ok(())
}

fn active_profile<'a>(
    atla_config: &'a AtlaConfig,
    global: &GlobalArgs,
) -> anyhow::Result<(&'a str, &'a Profile)> {
    atla_config
        .active_profile(config::active_profile(global))
        .ok_or_else(|| anyhow::anyhow!("no active profile; run `atla auth login` first"))
}

fn token_for_profile(profile_name: &str, profile: &Profile) -> anyhow::Result<String> {
    let credential = profile.credential_ref(profile_name);
    let token = KeyringCredentialStore::default()
        .get_token(&credential)
        .context("failed to read API token from keyring")?;

    token.ok_or_else(|| {
        anyhow::anyhow!("missing API token; run `atla auth login --profile {profile_name}`")
    })
}

fn print_projects(
    projects: &[JiraProject],
    total: Option<u64>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(projects),
        OutputFormat::Keys => {
            for project in projects {
                if let Some(key) = &project.key {
                    println!("{key}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,name,type,style,archived");
            for project in projects {
                println!(
                    "{},{},{},{},{}",
                    csv_cell(project.key.as_deref().unwrap_or_default()),
                    csv_cell(project.name.as_deref().unwrap_or_default()),
                    csv_cell(project.project_type_key.as_deref().unwrap_or_default()),
                    csv_cell(project.style.as_deref().unwrap_or_default()),
                    csv_cell(&project.archived.unwrap_or(false).to_string())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<12} {:<16} {:<12} NAME", "KEY", "TYPE", "STYLE");
            for project in projects {
                println!(
                    "{:<12} {:<16} {:<12} {}",
                    project.key.as_deref().unwrap_or("-"),
                    project.project_type_key.as_deref().unwrap_or("-"),
                    project.style.as_deref().unwrap_or("-"),
                    project.name.as_deref().unwrap_or("-")
                );
            }

            if let Some(total) = total {
                println!();
                println!("Showing {} of {total} projects.", projects.len());
            }
            Ok(())
        }
    }
}

fn print_issues(issues: &[JiraIssue], global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(issues),
        OutputFormat::Keys => {
            for issue in issues {
                if let Some(key) = &issue.key {
                    println!("{key}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,summary,status,assignee,type,priority,id");
            for issue in issues {
                println!(
                    "{},{},{},{},{},{},{}",
                    csv_cell(issue.key.as_deref().unwrap_or_default()),
                    csv_cell(issue.summary().unwrap_or_default()),
                    csv_cell(issue.status_name().unwrap_or_default()),
                    csv_cell(issue.assignee_display_name().unwrap_or_default()),
                    csv_cell(issue.issue_type_name().unwrap_or_default()),
                    csv_cell(issue.priority_name().unwrap_or_default()),
                    csv_cell(issue.id.as_deref().unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<14} {:<16} {:<20} SUMMARY", "KEY", "STATUS", "ASSIGNEE");
            for issue in issues {
                println!(
                    "{:<14} {:<16} {:<20} {}",
                    issue.key.as_deref().unwrap_or("-"),
                    issue.status_name().unwrap_or("-"),
                    issue.assignee_display_name().unwrap_or("-"),
                    issue.summary().unwrap_or("-")
                );
            }
            Ok(())
        }
    }
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
                csv_cell(issue.key.as_deref().unwrap_or_default()),
                csv_cell(issue.summary().unwrap_or_default()),
                csv_cell(issue.status_name().unwrap_or_default()),
                csv_cell(issue.assignee_display_name().unwrap_or_default()),
                csv_cell(issue.issue_type_name().unwrap_or_default()),
                csv_cell(issue.priority_name().unwrap_or_default()),
                csv_cell(issue.id.as_deref().unwrap_or_default())
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
                csv_cell(issue.key.as_deref().unwrap_or_default()),
                csv_cell(issue.id.as_deref().unwrap_or_default()),
                csv_cell(issue.self_url.as_deref().unwrap_or_default())
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
            println!("{},true", csv_cell(key));
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
            println!("{},true", csv_cell(key));
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
                csv_cell(key),
                csv_cell(user.account_id.as_deref().unwrap_or_default()),
                csv_cell(user.display_name.as_deref().unwrap_or_default())
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
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(transitions),
        OutputFormat::Keys => {
            for transition in transitions {
                if let Some(id) = &transition.id {
                    println!("{id}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,name,toStatus");
            for transition in transitions {
                println!(
                    "{},{},{}",
                    csv_cell(transition.id.as_deref().unwrap_or_default()),
                    csv_cell(transition.name.as_deref().unwrap_or_default()),
                    csv_cell(
                        transition
                            .to_status
                            .as_ref()
                            .and_then(|status| status.name.as_deref())
                            .unwrap_or_default()
                    )
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<8} {:<24} TO STATUS", "ID", "NAME");
            for transition in transitions {
                println!(
                    "{:<8} {:<24} {}",
                    transition.id.as_deref().unwrap_or("-"),
                    transition.name.as_deref().unwrap_or("-"),
                    transition
                        .to_status
                        .as_ref()
                        .and_then(|status| status.name.as_deref())
                        .unwrap_or("-")
                );
            }
            Ok(())
        }
    }
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
                csv_cell(key),
                csv_cell(transition.id.as_deref().unwrap_or_default()),
                csv_cell(transition.name.as_deref().unwrap_or_default()),
                csv_cell(
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
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(page),
        OutputFormat::Keys => {
            for comment in &page.comments {
                if let Some(id) = &comment.id {
                    println!("{id}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,author,created,updated,body");
            for comment in &page.comments {
                println!(
                    "{},{},{},{},{}",
                    csv_cell(comment.id.as_deref().unwrap_or_default()),
                    csv_cell(comment.author_display_name.as_deref().unwrap_or_default()),
                    csv_cell(comment.created.as_deref().unwrap_or_default()),
                    csv_cell(comment.updated.as_deref().unwrap_or_default()),
                    csv_cell(comment.body_text.as_deref().unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<12} {:<20} {:<22} BODY", "ID", "AUTHOR", "CREATED");
            for comment in &page.comments {
                println!(
                    "{:<12} {:<20} {:<22} {}",
                    comment.id.as_deref().unwrap_or("-"),
                    comment.author_display_name.as_deref().unwrap_or("-"),
                    comment.created.as_deref().unwrap_or("-"),
                    comment
                        .body_text
                        .as_deref()
                        .unwrap_or("-")
                        .replace('\n', " ")
                );
            }
            if let Some(total) = page.total {
                println!();
                println!("Showing {} of {total} comments.", page.comments.len());
            }
            Ok(())
        }
    }
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
                csv_cell(comment.id.as_deref().unwrap_or_default()),
                csv_cell(comment.author_display_name.as_deref().unwrap_or_default()),
                csv_cell(comment.created.as_deref().unwrap_or_default()),
                csv_cell(comment.updated.as_deref().unwrap_or_default()),
                csv_cell(comment.body_text.as_deref().unwrap_or_default())
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
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(page),
        OutputFormat::Keys => {
            for board in &page.values {
                if let Some(id) = board.id {
                    println!("{id}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,name,type,self");
            for board in &page.values {
                println!(
                    "{},{},{},{}",
                    csv_cell(&board.id.map(|id| id.to_string()).unwrap_or_default()),
                    csv_cell(board.name.as_deref().unwrap_or_default()),
                    csv_cell(board.board_type.as_deref().unwrap_or_default()),
                    csv_cell(board.self_url.as_deref().unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<8} {:<10} NAME", "ID", "TYPE");
            for board in &page.values {
                println!(
                    "{:<8} {:<10} {}",
                    board.id.map(|id| id.to_string()).unwrap_or("-".to_owned()),
                    board.board_type.as_deref().unwrap_or("-"),
                    board.name.as_deref().unwrap_or("-")
                );
            }
            if let Some(total) = page.total {
                println!();
                println!("Showing {} of {total} boards.", page.values.len());
            }
            Ok(())
        }
    }
}

fn print_sprints(page: &JiraSprintPage, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(page),
        OutputFormat::Keys => {
            for sprint in &page.values {
                if let Some(id) = sprint.id {
                    println!("{id}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,name,state,originBoardId,startDate,endDate,completeDate,goal");
            for sprint in &page.values {
                println!(
                    "{},{},{},{},{},{},{},{}",
                    csv_cell(&sprint.id.map(|id| id.to_string()).unwrap_or_default()),
                    csv_cell(sprint.name.as_deref().unwrap_or_default()),
                    csv_cell(sprint.state.as_deref().unwrap_or_default()),
                    csv_cell(
                        &sprint
                            .origin_board_id
                            .map(|id| id.to_string())
                            .unwrap_or_default()
                    ),
                    csv_cell(sprint.start_date.as_deref().unwrap_or_default()),
                    csv_cell(sprint.end_date.as_deref().unwrap_or_default()),
                    csv_cell(sprint.complete_date.as_deref().unwrap_or_default()),
                    csv_cell(sprint.goal.as_deref().unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<8} {:<10} {:<12} NAME", "ID", "STATE", "BOARD");
            for sprint in &page.values {
                println!(
                    "{:<8} {:<10} {:<12} {}",
                    sprint.id.map(|id| id.to_string()).unwrap_or("-".to_owned()),
                    sprint.state.as_deref().unwrap_or("-"),
                    sprint
                        .origin_board_id
                        .map(|id| id.to_string())
                        .unwrap_or("-".to_owned()),
                    sprint.name.as_deref().unwrap_or("-")
                );
            }
            if let Some(total) = page.total {
                println!();
                println!("Showing {} of {total} sprints.", page.values.len());
            }
            Ok(())
        }
    }
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
                csv_cell(&sprint.id.map(|id| id.to_string()).unwrap_or_default()),
                csv_cell(sprint.name.as_deref().unwrap_or_default()),
                csv_cell(sprint.state.as_deref().unwrap_or_default()),
                csv_cell(
                    &sprint
                        .origin_board_id
                        .map(|id| id.to_string())
                        .unwrap_or_default()
                ),
                csv_cell(sprint.start_date.as_deref().unwrap_or_default()),
                csv_cell(sprint.end_date.as_deref().unwrap_or_default()),
                csv_cell(sprint.complete_date.as_deref().unwrap_or_default()),
                csv_cell(sprint.goal.as_deref().unwrap_or_default())
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
                csv_cell(project.key.as_deref().unwrap_or_default()),
                csv_cell(project.name.as_deref().unwrap_or_default()),
                csv_cell(project.project_type_key.as_deref().unwrap_or_default()),
                csv_cell(project.style.as_deref().unwrap_or_default()),
                csv_cell(&project.archived.unwrap_or(false).to_string()),
                csv_cell(project.id.as_deref().unwrap_or_default())
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

fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
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
