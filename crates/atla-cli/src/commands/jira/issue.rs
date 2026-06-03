use anyhow::Context;
use atla_core::{
    JiraAssigneeTarget, JiraIssueAssign, JiraIssueCreate, JiraIssueFieldsQuery, JiraIssueList,
    JiraIssueUpdate, default_issue_fields,
};

use crate::cli::{GlobalArgs, IssueAction, IssueCommand, IssueCommentAction};
use crate::context::AppContext;

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

pub(super) async fn run_issue(command: IssueCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        IssueAction::List {
            project,
            status,
            issue_type,
            assignee,
            jql,
            limit,
            all,
            page_token,
            fields,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let requested_fields = parse_issue_fields(fields.as_deref())?;
            let max_results = if all { u32::MAX } else { limit.clamp(1, 5000) };
            let list = JiraIssueList {
                project_key: project,
                status,
                issue_type,
                assignee,
                jql,
                max_results,
                fields: requested_fields.clone(),
            };
            let mut search = list
                .to_search(profile.default_project.as_deref())
                .context("failed to build Jira issue list query")?;
            search.next_page_token = crate::pagination::decode_jira_jql_token(
                page_token.as_deref(),
                &search.jql,
                requested_fields.as_deref(),
            )?;

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

            let next_cli_token = if !all && matches!(page.is_last, Some(false)) {
                crate::pagination::jira_jql_next_token(
                    page.next_page_token.clone(),
                    &search.jql,
                    requested_fields.as_deref(),
                )?
            } else {
                None
            };
            let next_command = next_cli_token.as_ref().map(|token| {
                crate::pagination::jira_search_next_command(
                    &search.jql,
                    limit,
                    requested_fields.as_deref(),
                    token,
                )
            });

            match global.output.unwrap_or(crate::cli::OutputFormat::Table) {
                crate::cli::OutputFormat::Json => crate::output::print_json(&serde_json::json!({
                    "issues": page.issues,
                    "pagination": {
                        "isLast": page.is_last.unwrap_or(true),
                        "nextPageToken": next_cli_token,
                        "nextCommand": next_command,
                    }
                }))?,
                crate::cli::OutputFormat::Table => {
                    let footer = next_command
                        .as_deref()
                        .map(crate::pagination::next_page_footer);
                    print_issues_with_footer(
                        &page.issues,
                        global,
                        requested_fields.as_deref(),
                        footer,
                    )?;
                }
                crate::cli::OutputFormat::Csv | crate::cli::OutputFormat::Keys => {
                    print_issues(&page.issues, global, requested_fields.as_deref())?;
                    if let Some(command) = next_command {
                        eprintln!("{}", crate::pagination::next_page_footer(&command));
                    }
                }
            }
        }
        IssueAction::Create {
            project,
            issue_type,
            summary,
            description,
            description_file,
            fields,
            labels,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let mut parsed_fields = parse_fields(&fields)?;
            if let Some(labels) = labels {
                let values = labels
                    .split(',')
                    .map(str::trim)
                    .filter(|label| !label.is_empty())
                    .map(|label| serde_json::Value::String(label.to_owned()))
                    .collect::<Vec<_>>();
                parsed_fields.insert("labels".to_owned(), serde_json::Value::Array(values));
            }
            let issue = JiraIssueCreate {
                project_key: project,
                issue_type,
                summary,
                description: read_optional_text(description, description_file.as_deref())?,
                fields: parsed_fields,
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
        IssueAction::View {
            key,
            web,
            fields,
            with_github,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let requested_fields = parse_issue_fields(fields.as_deref())?;

            // When no --fields specified, always fetch description + issuelinks in addition to defaults.
            let fetch_fields = requested_fields.clone().or_else(|| {
                let mut f = default_issue_fields();
                f.push("description".to_owned());
                f.push("issuelinks".to_owned());
                Some(f)
            });

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
                    issue_fields_for_url(fetch_fields.as_deref())
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
            let output_format = global.output.unwrap_or(crate::cli::OutputFormat::Table);

            // Issue 2: for JSON + --with-github, fetch everything concurrently and
            // emit a single combined JSON object instead of multiple separate payloads.
            if with_github && output_format == crate::cli::OutputFormat::Json {
                let (issue, prs, commits) = tokio::try_join!(
                    client.get_issue(&key, fetch_fields),
                    client.list_github_pull_requests(&key),
                    client.list_github_commits(&key),
                )
                .with_context(|| {
                    format!(
                        "failed to fetch data for `{key}` from {}",
                        client.instance_url()
                    )
                })?;
                return print_issue_with_github(&issue, &prs, &commits);
            }

            let issue = client
                .get_issue(&key, fetch_fields)
                .await
                .with_context(|| {
                    format!(
                        "failed to load Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;

            print_issue(&issue, global, requested_fields.as_deref())?;

            // In table mode with no custom --fields, also show comments
            if (global.output.is_none() || global.output == Some(crate::cli::OutputFormat::Table))
                && requested_fields.is_none()
            {
                let comment_page = client.list_comments(&key, 50).await.with_context(|| {
                    format!(
                        "failed to load comments for Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
                if !comment_page.comments.is_empty() {
                    print_issue_comments_section(&comment_page, global)?;
                }
            }

            if with_github {
                let (prs, commits) = tokio::try_join!(
                    client.list_github_pull_requests(&key),
                    client.list_github_commits(&key),
                )
                .with_context(|| {
                    format!(
                        "failed to fetch GitHub data for `{key}` from {}",
                        client.instance_url()
                    )
                })?;

                let is_table = global.output.is_none()
                    || global.output == Some(crate::cli::OutputFormat::Table);
                if is_table {
                    print_section_header("GitHub Pull Requests");
                }
                print_github_pull_requests(&prs, global)?;
                if is_table {
                    print_section_header("GitHub Commits");
                }
                print_github_commits(&commits, global)?;
            }
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
            target,
            account_id,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let target = if target.unassign {
                JiraAssigneeTarget::Unassign
            } else if let Some(to) = target.to {
                if to.eq_ignore_ascii_case("me") {
                    JiraAssigneeTarget::Me
                } else if account_id {
                    JiraAssigneeTarget::AccountId(to)
                } else {
                    JiraAssigneeTarget::Query(to)
                }
            } else {
                anyhow::bail!("provide --to <user> or --unassign");
            };
            let is_unassign = matches!(&target, JiraAssigneeTarget::Unassign);
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
                if is_unassign {
                    println!("Would PUT {url} (unassign) using profile `{profile_name}`");
                } else {
                    println!("Would PUT {url} using profile `{profile_name}`");
                }
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
                } else if can_prompt(global) {
                    anyhow::bail!("no transitions available for issue `{key}`");
                } else {
                    let names: Vec<_> = transitions
                        .iter()
                        .filter_map(|t| t.name.as_deref())
                        .collect();
                    anyhow::bail!(
                        "--to is required in non-interactive mode; available transitions: {}",
                        names.join(", ")
                    );
                }
            }
        }
        IssueAction::Comment { action } => match action {
            IssueCommentAction::Add {
                key,
                body,
                body_flag,
                body_file,
            } => {
                let ctx = AppContext::load(global)?;
                let profile_name = ctx.profile_name();
                let profile = ctx.profile();
                let body =
                    read_required_text(body.or(body_flag), body_file.as_deref(), "comment body")?;

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
            IssueCommentAction::List {
                key,
                limit,
                all,
                page_token,
            } => {
                let ctx = AppContext::load(global)?;
                let profile_name = ctx.profile_name();
                let profile = ctx.profile();
                let max_results = if all { u32::MAX } else { limit.clamp(1, 1000) };
                let query_hash =
                    crate::pagination::query_hash("jira.comment.list", &[("key", key.clone())]);
                let start_at = crate::pagination::decode_jira_offset_token(
                    page_token.as_deref(),
                    "jira.comment.list",
                    query_hash.clone(),
                )?;

                if global.dry_run {
                    let url = format!(
                        "{}/rest/api/3/issue/{}/comment?startAt=0&maxResults={max_results}",
                        profile.instance.trim_end_matches('/'),
                        key
                    );
                    println!("Would GET {url} using profile `{profile_name}`");
                    return Ok(());
                }

                let client = ctx.jira_client()?;
                let page = client
                    .list_comments_from(&key, max_results, start_at)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to list comments for Jira issue `{key}` from {}",
                            client.instance_url()
                        )
                    })?;

                let next_start = (!all
                    && page.total.is_some_and(|total| {
                        start_at + (page.comments.len() as u64) < total as u64
                    }))
                .then_some(start_at + page.comments.len() as u64);
                let next_cli_token = crate::pagination::jira_offset_next_token(
                    "jira.comment.list",
                    next_start,
                    query_hash,
                )?;
                let next_command = next_cli_token.as_ref().map(|token| {
                    crate::pagination::next_command(
                        vec![
                            "atla".to_owned(),
                            "jira".to_owned(),
                            "issue".to_owned(),
                            "comment".to_owned(),
                            "list".to_owned(),
                            crate::pagination::quote(&key),
                        ],
                        limit,
                        token,
                    )
                });
                match global.output.unwrap_or(crate::cli::OutputFormat::Table) {
                    crate::cli::OutputFormat::Json => crate::output::print_json(
                        &serde_json::json!({"comments": page.comments, "total": page.total, "pagination": {"isLast": next_cli_token.is_none(), "nextPageToken": next_cli_token, "nextCommand": next_command}}),
                    )?,
                    crate::cli::OutputFormat::Table => print_comments_with_footer(
                        &page,
                        global,
                        next_command
                            .as_deref()
                            .map(crate::pagination::next_page_footer),
                    )?,
                    crate::cli::OutputFormat::Csv | crate::cli::OutputFormat::Keys => {
                        print_comments(&page, global)?;
                        if let Some(command) = next_command {
                            eprintln!("{}", crate::pagination::next_page_footer(&command));
                        }
                    }
                }
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
        IssueAction::Fields {
            project,
            issue_type,
            required_only,
        } => {
            let ctx = AppContext::load(global)?;
            let client = ctx.jira_client()?;

            let issue_types = client
                .list_issue_types(&project)
                .await
                .with_context(|| format!("failed to list issue types for project `{project}`"))?;

            let matched = issue_types
                .iter()
                .find(|t| {
                    t.name
                        .as_deref()
                        .map(|n| n.eq_ignore_ascii_case(&issue_type))
                        == Some(true)
                        || t.id.as_deref() == Some(issue_type.as_str())
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("issue type `{issue_type}` not found in project `{project}`")
                })?;

            let type_id = matched
                .id
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("issue type has no id"))?;

            let query = JiraIssueFieldsQuery {
                project_key: project.clone(),
                issue_type_id: type_id.to_owned(),
            };

            let fields = client.get_issue_fields(&query).await.with_context(|| {
                format!("failed to get fields for `{issue_type}` in project `{project}`")
            })?;

            print_issue_fields(&fields, required_only, global)?;
        }
    }

    Ok(())
}
