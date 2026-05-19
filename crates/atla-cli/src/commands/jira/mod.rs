use anyhow::Context;
use atla_core::{
    JiraAssigneeTarget, JiraAttachmentDownload, JiraBoardPage, JiraBoardSearch, JiraComment,
    JiraCommentPage, JiraCreatedIssue, JiraIssue, JiraIssueAssign, JiraIssueCreate,
    JiraIssueLabelUpdate, JiraIssueLink, JiraIssueLinkCreate, JiraIssueList, JiraIssueSearch,
    JiraIssueType, JiraIssueUpdate, JiraProject, JiraProjectSearch, JiraSprint, JiraSprintCreate,
    JiraSprintPage, JiraSprintSearch, JiraSprintUpdate, JiraTransition, JiraUser, JiraWorklog,
    JiraWorklogCreate, JiraWorklogPage,
};
use dialoguer::Select;
use std::io::{IsTerminal, stdin, stdout};
use std::path::Path;

use crate::cli::{
    BoardAction, BoardCommand, GlobalArgs, IssueAction, IssueAttachmentAction, IssueCommand,
    IssueCommentAction, IssueLinkAction, IssueWorklogAction, JiraCommand, JiraResource,
    OutputFormat, ProjectAction, ProjectCommand, SprintAction, SprintCommand,
};
use crate::context::AppContext;
use crate::output;

pub async fn run(command: JiraCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        JiraResource::Issue(command) => run_issue(command, global).await?,
        JiraResource::Project(command) => run_project(command, global).await?,
        JiraResource::Sprint(command) => run_sprint(command, global).await?,
        JiraResource::Board(command) => run_board(command, global).await?,
        JiraResource::Search { jql, limit, fields } => {
            run_search(jql, limit, fields, global).await?
        }
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
            fields,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let requested_fields = parse_issue_fields(fields.as_deref())?;
            let list = JiraIssueList {
                project_key: project,
                status,
                assignee,
                jql,
                max_results: limit.clamp(1, 5000),
                fields: requested_fields.clone(),
            };
            let search = list
                .to_search(profile.default_project.as_deref())
                .context("failed to build Jira issue list query")?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/search/jql?maxResults={}&fields={}",
                    profile.instance.trim_end_matches('/'),
                    search.max_results,
                    issue_fields_for_url(requested_fields.as_deref())
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

            print_issues(&page.issues, global, requested_fields.as_deref())?;
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
        IssueAction::View { key, web, fields } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let requested_fields = parse_issue_fields(fields.as_deref())?;

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
                    "{}/rest/api/3/issue/{}?fields={}",
                    profile.instance.trim_end_matches('/'),
                    key,
                    issue_fields_for_url(requested_fields.as_deref())
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
            let issue = client
                .get_issue(&key, requested_fields.clone())
                .await
                .with_context(|| {
                    format!(
                        "failed to load Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

            print_issue(&issue, global, requested_fields.as_deref())?;
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
        IssueAction::Transition { key, to, fields } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let fields = parse_fields(&fields)?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/issue/{}/transitions",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                if let Some(to) = &to {
                    println!(
                        "Would GET {url}?expand=transitions.fields, then POST transition `{to}` using profile `{profile_name}`"
                    );
                } else {
                    println!(
                        "Would GET {url}?expand=transitions.fields using profile `{profile_name}`"
                    );
                }
                return Ok(());
            }

            let client = ctx.jira_client()?;
            if let Some(to) = to {
                let transition = client
                    .transition_issue(&key, &to, fields)
                    .await
                    .with_context(|| {
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
                        .transition_issue(&key, transition_id, fields)
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
            IssueCommentAction::Update {
                key,
                comment_id,
                body,
                body_file,
            } => {
                let ctx = AppContext::load(global)?;
                let profile_name = ctx.profile_name();
                let profile = ctx.profile();
                let body = read_required_text(body, body_file.as_deref(), "comment body")?;

                if global.dry_run {
                    println!(
                        "Would PUT {}/rest/api/3/issue/{}/comment/{} using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/'),
                        key,
                        comment_id
                    );
                    return Ok(());
                }

                let client = ctx.jira_client()?;
                let comment = client
                    .update_comment(&key, &comment_id, &body)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to update comment `{comment_id}` on Jira issue `{key}` from {}",
                            client.instance_url()
                        )
                    })?;

                print_comment(&comment, global)?;
            }
            IssueCommentAction::Delete {
                key,
                comment_id,
                yes,
            } => {
                let ctx = AppContext::load(global)?;
                let profile_name = ctx.profile_name();
                let profile = ctx.profile();

                if !yes && !global.dry_run {
                    anyhow::bail!("refusing to delete Jira comment `{comment_id}` without --yes");
                }

                if global.dry_run {
                    println!(
                        "Would DELETE {}/rest/api/3/issue/{}/comment/{} using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/'),
                        key,
                        comment_id
                    );
                    return Ok(());
                }

                let client = ctx.jira_client()?;
                client
                    .delete_comment(&key, &comment_id)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to delete comment `{comment_id}` on Jira issue `{key}` from {}",
                            client.instance_url()
                        )
                    })?;
                print_deleted("comment", &comment_id, global)?;
            }
        },
        IssueAction::Attachment { action } => run_issue_attachment(action, global).await?,
        IssueAction::Link { action } => run_issue_link(action, global).await?,
        IssueAction::Worklog { action } => run_issue_worklog(action, global).await?,
    }

    Ok(())
}

async fn run_search(
    jql: String,
    limit: u32,
    fields: Option<String>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let requested_fields = parse_issue_fields(fields.as_deref())?;
    let search = JiraIssueSearch {
        jql,
        max_results: limit.clamp(1, 5000),
        fields: requested_fields.clone(),
    };

    if global.dry_run {
        let url = format!(
            "{}/rest/api/3/search/jql?maxResults={}&fields={}",
            profile.instance.trim_end_matches('/'),
            search.max_results,
            issue_fields_for_url(requested_fields.as_deref())
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

    print_issues(&page.issues, global, requested_fields.as_deref())?;
    Ok(())
}

async fn run_issue_attachment(
    action: IssueAttachmentAction,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match action {
        IssueAttachmentAction::Download {
            target,
            all,
            output,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                if all {
                    println!(
                        "Would GET {}/rest/api/3/issue/{}?fields=attachment, then download each attachment using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/'),
                        target
                    );
                } else {
                    println!(
                        "Would GET {}/rest/api/3/attachment/{}, then download its content using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/'),
                        target
                    );
                }
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let downloads = if all {
                client
                    .download_issue_attachments(&target, output.as_deref())
                    .await
                    .with_context(|| {
                        format!(
                            "failed to download Jira issue attachments for `{target}` from {}",
                            client.instance_url()
                        )
                    })?
            } else {
                vec![
                    client
                        .download_attachment(&target, output.as_deref())
                        .await
                        .with_context(|| {
                            format!(
                                "failed to download Jira attachment `{target}` from {}",
                                client.instance_url()
                            )
                        })?,
                ]
            };

            print_attachment_downloads(&downloads, global)?;
        }
    }

    Ok(())
}

async fn run_issue_link(action: IssueLinkAction, global: &GlobalArgs) -> anyhow::Result<()> {
    match action {
        IssueLinkAction::Add {
            key,
            link_type,
            target,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/api/3/issueLink using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .create_issue_link(&JiraIssueLinkCreate {
                    source_key: key.clone(),
                    target_key: target.clone(),
                    link_type,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to link Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
            print_issue_update(&key, global)?;
        }
        IssueLinkAction::List { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/rest/api/3/issue/{}?fields=issuelinks using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let links = client.list_issue_links(&key).await.with_context(|| {
                format!(
                    "failed to list links for Jira issue `{key}` from {}",
                    client.instance_url()
                )
            })?;
            print_issue_links(&links, global)?;
        }
        IssueLinkAction::Remove { link_id, yes } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if !yes && !global.dry_run {
                anyhow::bail!("refusing to delete Jira issue link `{link_id}` without --yes");
            }

            if global.dry_run {
                println!(
                    "Would DELETE {}/rest/api/3/issueLink/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    link_id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client.delete_issue_link(&link_id).await.with_context(|| {
                format!(
                    "failed to delete Jira issue link `{link_id}` from {}",
                    client.instance_url()
                )
            })?;
            print_deleted("issueLink", &link_id, global)?;
        }
    }

    Ok(())
}

async fn run_issue_worklog(action: IssueWorklogAction, global: &GlobalArgs) -> anyhow::Result<()> {
    match action {
        IssueWorklogAction::Add { key, time, comment } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/api/3/issue/{}/worklog using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let worklog = client
                .add_worklog(&JiraWorklogCreate {
                    issue_id_or_key: key.clone(),
                    time_spent: time,
                    comment,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to add worklog to Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
            print_worklog(&worklog, global)?;
        }
        IssueWorklogAction::List { key, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let limit = limit.clamp(1, 1000);

            if global.dry_run {
                println!(
                    "Would GET {}/rest/api/3/issue/{}/worklog?startAt=0&maxResults={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key,
                    limit
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let page = client.list_worklogs(&key, limit).await.with_context(|| {
                format!(
                    "failed to list worklogs for Jira issue `{key}` from {}",
                    client.instance_url()
                )
            })?;
            print_worklogs(&page, global)?;
        }
    }

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
        ProjectAction::IssueTypes { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/rest/api/3/project/{} then GET /rest/api/3/issuetype/project using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let issue_types = client.list_issue_types(&key).await.with_context(|| {
                format!(
                    "failed to list issue types for Jira project `{key}` from {}",
                    client.instance_url()
                )
            })?;
            print_issue_types(&issue_types, global)?;
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
        SprintAction::Create {
            board,
            name,
            start,
            end,
            goal,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/agile/1.0/sprint using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let sprint = client
                .create_sprint(&JiraSprintCreate {
                    board_id: board,
                    name,
                    start_date: start,
                    end_date: end,
                    goal,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to create Jira sprint from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint(&sprint, global)?;
        }
        SprintAction::Start { id, start, end } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would PUT {}/rest/agile/1.0/sprint/{} with state active using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let sprint = client
                .update_sprint(&JiraSprintUpdate {
                    id,
                    state: Some("active".to_owned()),
                    name: None,
                    start_date: start,
                    end_date: end,
                    goal: None,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to start Jira sprint `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint(&sprint, global)?;
        }
        SprintAction::Close { id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would PUT {}/rest/agile/1.0/sprint/{} with state closed using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            let sprint = client
                .update_sprint(&JiraSprintUpdate {
                    id,
                    state: Some("closed".to_owned()),
                    name: None,
                    start_date: None,
                    end_date: None,
                    goal: None,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to close Jira sprint `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint(&sprint, global)?;
        }
        SprintAction::Add { id, issues } => {
            if issues.is_empty() {
                anyhow::bail!("provide at least one issue with --issues");
            }
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/rest/agile/1.0/sprint/{}/issue using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .move_issues_to_sprint(id, &issues)
                .await
                .with_context(|| {
                    format!(
                        "failed to move issues to Jira sprint `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint_issue_move(id, &issues, global)?;
        }
        SprintAction::Remove { id, issue } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let issues = vec![issue];

            if global.dry_run {
                println!(
                    "Would POST {}/rest/agile/1.0/backlog/issue for sprint `{id}` using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.jira_client()?;
            client
                .move_issues_to_backlog(&issues)
                .await
                .with_context(|| {
                    format!(
                        "failed to remove issue from Jira sprint `{id}` via backlog move from {}",
                        client.instance_url()
                    )
                })?;
            print_sprint_issue_move(id, &issues, global)?;
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

fn print_issues(
    issues: &[JiraIssue],
    global: &GlobalArgs,
    requested_fields: Option<&[String]>,
) -> anyhow::Result<()> {
    let extra_fields = display_extra_issue_fields(requested_fields);
    let mut headers = vec![
        "key", "summary", "status", "assignee", "type", "priority", "id",
    ];
    headers.extend(extra_fields.iter().map(String::as_str));
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        issues,
        issues
            .iter()
            .filter_map(|issue| issue.key.clone())
            .collect(),
        &headers,
        issues
            .iter()
            .map(|issue| {
                let mut row = vec![
                    issue.key.as_deref().unwrap_or("-").to_owned(),
                    issue.summary().unwrap_or("-").to_owned(),
                    issue.status_name().unwrap_or("-").to_owned(),
                    issue.assignee_display_name().unwrap_or("-").to_owned(),
                    issue.issue_type_name().unwrap_or("-").to_owned(),
                    issue.priority_name().unwrap_or("-").to_owned(),
                    issue.id.as_deref().unwrap_or("-").to_owned(),
                ];
                row.extend(
                    extra_fields
                        .iter()
                        .map(|field| issue_field_cell(issue, field)),
                );
                row
            })
            .collect(),
        None,
    )
}

fn print_issue(
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
            for field in extra_fields {
                println!("{field}: {}", issue_field_cell(issue, &field));
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
        &["id", "name", "toStatus", "requiredFields"],
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
                    transition.required_fields().join(", "),
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

fn print_issue_links(links: &[JiraIssueLink], global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_worklogs(page: &JiraWorklogPage, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_worklog(worklog: &JiraWorklog, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_issue_types(types: &[JiraIssueType], global: &GlobalArgs) -> anyhow::Result<()> {
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

fn print_attachment_downloads(
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

fn print_sprint_issue_move(
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

fn print_deleted(kind: &str, id: &str, global: &GlobalArgs) -> anyhow::Result<()> {
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

fn parse_issue_fields(fields: Option<&str>) -> anyhow::Result<Option<Vec<String>>> {
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

fn issue_fields_for_url(fields: Option<&[String]>) -> String {
    fields
        .filter(|fields| !fields.is_empty())
        .map(|fields| fields.join(","))
        .unwrap_or_else(|| "summary,status,assignee,issuetype,priority".to_owned())
}

fn display_extra_issue_fields(fields: Option<&[String]>) -> Vec<String> {
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

fn issue_field_cell(issue: &JiraIssue, field: &str) -> String {
    issue
        .fields
        .get(field)
        .map(value_cell)
        .unwrap_or_else(|| "-".to_owned())
}

fn value_cell(value: &serde_json::Value) -> String {
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
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| serde_json::to_string(value).unwrap_or_else(|_| "-".to_owned())),
    }
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
