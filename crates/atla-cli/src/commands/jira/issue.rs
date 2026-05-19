use anyhow::Context;
use atla_core::{
    JiraAssigneeTarget, JiraIssueAssign, JiraIssueCreate, JiraIssueList, JiraIssueUpdate,
    default_issue_fields,
};

use crate::cli::{GlobalArgs, IssueAction, IssueCommand, IssueCommentAction};
use crate::context::AppContext;

use super::attachment::run_issue_attachment;
use super::format::{
    can_prompt, issue_fields_for_url, open_web_url, parse_fields, parse_issue_fields,
    parse_label_update, print_comment, print_comments, print_created_issue, print_deleted,
    print_issue, print_issue_assign, print_issue_comments_section, print_issue_delete,
    print_issue_update, print_issues, print_transition_update, read_optional_text,
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
            fields,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let requested_fields = parse_issue_fields(fields.as_deref())?;
            let list = JiraIssueList {
                project_key: project,
                status,
                issue_type,
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
        IssueAction::View { key, web, fields } => {
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
            unassign,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let target = if unassign {
                JiraAssigneeTarget::Unassign
            } else if let Some(to) = to {
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
                if unassign {
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
