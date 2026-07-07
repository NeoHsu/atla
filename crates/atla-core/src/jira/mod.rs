use atla_jira_api::Client as GeneratedClient;

use crate::client::{ApiError, AtlassianClient, read_json};

mod attachments;
mod boards;
mod comments;
mod issues;
mod links;
pub mod models;
mod projects;
mod sprints;
pub mod util;
mod worklogs;

pub use comments::comment_request_body;
pub use models::*;

pub fn default_issue_fields() -> Vec<String> {
    [
        "summary",
        "status",
        "assignee",
        "issuetype",
        "priority",
        "labels",
        "created",
        "updated",
        "reporter",
        "parent",
        "subtasks",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[derive(Debug, Clone)]
pub struct JiraClient {
    raw_client: AtlassianClient,
    generated: GeneratedClient,
}

impl JiraClient {
    pub fn new(client: AtlassianClient) -> Self {
        let http_client = client.authed_http_client();
        let generated = GeneratedClient::new_with_client(&client.instance().base_url, http_client);

        Self {
            raw_client: client,
            generated,
        }
    }

    pub fn instance_url(&self) -> &str {
        &self.raw_client.instance().base_url
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<JiraUser>, ApiError> {
        read_json(
            self.raw_client
                .get("/rest/api/3/user/search")
                .query(&[("query", query), ("maxResults", "50")]),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::util::adf_body;
    use super::*;
    use atla_jira_api::types as generated_types;

    #[cfg(test)]
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct JiraTransitionPage {
        #[serde(default)]
        transitions: Vec<JiraTransition>,
    }

    #[test]
    fn parses_project_search_page() {
        let page: JiraProjectPage = serde_json::from_str(
            r#"{
                "startAt": 0,
                "maxResults": 50,
                "total": 1,
                "isLast": true,
                "values": [
                    {
                        "id": "10000",
                        "key": "PROJ",
                        "name": "Project",
                        "projectTypeKey": "software",
                        "style": "classic",
                        "simplified": false,
                        "archived": false
                    }
                ]
            }"#,
        )
        .expect("parse project page");

        assert_eq!(page.values[0].key.as_deref(), Some("PROJ"));
        assert_eq!(page.total, Some(1));
    }

    #[test]
    fn parses_project_detail() {
        let project: JiraProject = serde_json::from_str(
            r#"{
                "id": "10000",
                "key": "PROJ",
                "name": "Project",
                "projectTypeKey": "software",
                "style": "classic",
                "simplified": false,
                "archived": false
            }"#,
        )
        .expect("parse project detail");

        assert_eq!(project.id.as_deref(), Some("10000"));
        assert_eq!(project.key.as_deref(), Some("PROJ"));
        assert_eq!(project.project_type_key.as_deref(), Some("software"));
    }

    #[test]
    fn converts_generated_project_page() {
        let page = generated_types::PageBeanProject {
            start_at: Some(0),
            max_results: Some(50),
            total: Some(1),
            is_last: Some(true),
            values: vec![generated_types::Project {
                id: Some("10000".to_owned()),
                key: Some("PROJ".to_owned()),
                name: Some("Project".to_owned()),
                project_type_key: Some(generated_types::ProjectProjectTypeKey::Software),
                style: Some(generated_types::ProjectStyle::Classic),
                simplified: Some(false),
                archived: Some(false),
            }],
        };

        let page = JiraProjectPage::from(page);

        assert_eq!(page.total, Some(1));
        assert_eq!(page.values[0].key.as_deref(), Some("PROJ"));
        assert_eq!(page.values[0].project_type_key.as_deref(), Some("software"));
        assert_eq!(page.values[0].style.as_deref(), Some("classic"));
    }

    #[test]
    fn builds_generated_issue_create_request() {
        let issue = JiraIssueCreate {
            project_key: "PROJ".to_owned(),
            issue_type: "Task".to_owned(),
            summary: "Fix login".to_owned(),
            description: Some("Line one\nLine two".to_owned()),
            fields: serde_json::Map::from_iter([(
                "priority".to_owned(),
                serde_json::json!({ "name": "High" }),
            )]),
        };

        let generated = issue.to_generated();
        let fields = &generated.fields;

        assert_eq!(fields["project"], serde_json::json!({ "key": "PROJ" }));
        assert_eq!(fields["issuetype"], serde_json::json!({ "name": "Task" }));
        assert_eq!(fields["summary"], serde_json::json!("Fix login"));
        assert_eq!(fields["priority"], serde_json::json!({ "name": "High" }));
        assert_eq!(fields["description"]["type"], serde_json::json!("doc"));
        assert_eq!(
            fields["description"]["content"].as_array().unwrap().len(),
            1
        );
    }

    #[test]
    fn builds_generated_issue_update_request() {
        let issue = JiraIssueUpdate {
            issue_id_or_key: "PROJ-1".to_owned(),
            summary: Some("Updated summary".to_owned()),
            description: None,
            fields: serde_json::Map::from_iter([("labels".to_owned(), serde_json::json!(["cli"]))]),
        };

        let generated = issue.to_generated();
        let fields = &generated.fields;

        assert_eq!(fields["summary"], serde_json::json!("Updated summary"));
        assert_eq!(fields["labels"], serde_json::json!(["cli"]));
        assert_eq!(fields.get("description"), None);
    }

    #[test]
    fn builds_label_update_request() {
        let update = JiraIssueLabelUpdate {
            issue_id_or_key: "PROJ-1".to_owned(),
            add: vec!["urgent".to_owned()],
            remove: vec!["low".to_owned()],
        };

        assert_eq!(
            update.to_json(),
            serde_json::json!({
                "update": {
                    "labels": [
                        { "add": "urgent" },
                        { "remove": "low" }
                    ]
                }
            })
        );
    }

    #[test]
    fn builds_issue_list_jql_from_filters() {
        let list = JiraIssueList {
            project_key: None,
            status: Some("In Progress".to_owned()),
            issue_type: None,
            assignee: Some("me".to_owned()),
            jql: None,
            max_results: 25,
            fields: None,
        };

        let search = list.to_search(Some("PROJ")).expect("search");

        assert_eq!(
            search.jql,
            "project = \"PROJ\" AND status = \"In Progress\" AND assignee = currentUser() ORDER BY updated DESC"
        );
        assert_eq!(search.max_results, 25);
        assert_eq!(search.issue_fields(), default_issue_fields());
    }

    #[test]
    fn explicit_jql_overrides_issue_list_filters() {
        let list = JiraIssueList {
            project_key: Some("PROJ".to_owned()),
            status: Some("Done".to_owned()),
            issue_type: None,
            assignee: None,
            jql: Some("status = Open".to_owned()),
            max_results: 10,
            fields: None,
        };

        let search = list.to_search(None).expect("search");

        assert_eq!(search.jql, "project = \"PROJ\" AND (status = Open)");
        assert_eq!(search.max_results, 10);
    }

    #[test]
    fn issue_search_uses_requested_fields() {
        let search = JiraIssueSearch {
            jql: "project = PROJ".to_owned(),
            max_results: 10,
            fields: Some(vec!["summary".to_owned(), "attachment".to_owned()]),
            next_page_token: None,
        };

        assert_eq!(
            search.issue_fields(),
            vec!["summary".to_owned(), "attachment".to_owned()]
        );
    }

    #[test]
    fn converts_transition() {
        let transition = JiraTransition::from(generated_types::Transition {
            id: Some("31".to_owned()),
            name: Some("Done".to_owned()),
            to: Some(generated_types::Status {
                id: Some("10001".to_owned()),
                name: Some("Done".to_owned()),
            }),
        });

        assert_eq!(transition.id.as_deref(), Some("31"));
        assert_eq!(transition.name.as_deref(), Some("Done"));
        assert_eq!(
            transition
                .to_status
                .as_ref()
                .and_then(|status| status.name.as_deref()),
            Some("Done")
        );
    }

    #[test]
    fn parses_transition_metadata_required_fields() {
        let page: JiraTransitionPage = serde_json::from_value(serde_json::json!({
            "transitions": [{
                "id": "41",
                "name": "Validation",
                "to": { "id": "10002", "name": "Validation" },
                "fields": {
                    "customfield_12345": { "required": true, "name": "QA note" },
                    "customfield_67890": { "required": false, "name": "Optional date" }
                }
            }]
        }))
        .expect("transition metadata");

        let transition = &page.transitions[0];
        assert_eq!(transition.name.as_deref(), Some("Validation"));
        assert_eq!(
            transition
                .to_status
                .as_ref()
                .and_then(|status| status.name.as_deref()),
            Some("Validation")
        );
        assert_eq!(transition.required_fields(), vec!["customfield_12345"]);
    }

    #[test]
    fn converts_comment_body_to_plain_text() {
        let comment = JiraComment::from(generated_types::Comment {
            id: Some("10010".to_owned()),
            self_: None,
            body: adf_body("Line one\nLine two"),
            author: Some(generated_types::User {
                account_id: Some("account-id".to_owned()),
                display_name: Some("Neo".to_owned()),
                active: Some(true),
            }),
            created: Some("2026-05-18T00:00:00.000+0000".to_owned()),
            updated: None,
        });

        assert_eq!(comment.id.as_deref(), Some("10010"));
        assert_eq!(comment.body_text.as_deref(), Some("Line one Line two"));
        assert_eq!(comment.author_display_name.as_deref(), Some("Neo"));
    }

    #[test]
    fn converts_adf_nodes_using_attrs_text() {
        let comment = JiraComment::from(generated_types::Comment {
            id: Some("10011".to_owned()),
            self_: None,
            body: serde_json::json!({
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "mention",
                                "attrs": {
                                    "id": "abc",
                                    "text": "@Neo"
                                }
                            }
                        ]
                    }
                ]
            })
            .as_object()
            .expect("ADF root object")
            .clone()
            .into_iter()
            .collect(),
            author: None,
            created: None,
            updated: None,
        });

        assert_eq!(comment.body_text.as_deref(), Some("@Neo"));
    }

    #[test]
    fn parses_board_page() {
        let page: JiraBoardPage = serde_json::from_str(
            r#"{
                "isLast": false,
                "maxResults": 2,
                "startAt": 0,
                "total": 5,
                "values": [
                    {
                        "id": 84,
                        "name": "scrum board",
                        "self": "https://example.atlassian.net/rest/agile/1.0/board/84",
                        "type": "scrum"
                    }
                ]
            }"#,
        )
        .expect("parse board page");

        assert_eq!(page.total, Some(5));
        assert_eq!(page.values[0].id, Some(84));
        assert_eq!(page.values[0].board_type.as_deref(), Some("scrum"));
    }

    #[test]
    fn parses_sprint_page() {
        let page: JiraSprintPage = serde_json::from_str(
            r#"{
                "isLast": false,
                "maxResults": 2,
                "startAt": 0,
                "total": 5,
                "values": [
                    {
                        "id": 37,
                        "self": "https://example.atlassian.net/rest/agile/1.0/sprint/37",
                        "state": "active",
                        "name": "Sprint 1",
                        "startDate": "2026-05-18T00:00:00.000+0000",
                        "endDate": "2026-06-01T00:00:00.000+0000",
                        "originBoardId": 84,
                        "goal": "Ship the CLI"
                    }
                ]
            }"#,
        )
        .expect("parse sprint page");

        let sprint = &page.values[0];
        assert_eq!(sprint.id, Some(37));
        assert_eq!(sprint.state.as_deref(), Some("active"));
        assert_eq!(sprint.origin_board_id, Some(84));
        assert_eq!(sprint.goal.as_deref(), Some("Ship the CLI"));
    }

    #[test]
    fn parses_issue_search_page() {
        let page: JiraIssueSearchPage = serde_json::from_str(
            r#"{
                "isLast": true,
                "issues": [
                    {
                        "id": "10002",
                        "key": "PROJ-1",
                        "fields": {
                            "summary": "Fix login",
                            "status": { "name": "In Progress" },
                            "assignee": { "displayName": "Neo" },
                            "issuetype": { "name": "Bug" },
                            "priority": { "name": "High" }
                        }
                    }
                ]
            }"#,
        )
        .expect("parse issue search page");

        let issue = &page.issues[0];
        assert_eq!(issue.key.as_deref(), Some("PROJ-1"));
        assert_eq!(issue.summary(), Some("Fix login"));
        assert_eq!(issue.status_name(), Some("In Progress"));
        assert_eq!(issue.assignee_display_name(), Some("Neo"));
        assert_eq!(issue.issue_type_name(), Some("Bug"));
        assert_eq!(issue.priority_name(), Some("High"));
    }

    #[test]
    fn converts_generated_issue_search_page() {
        let fields: serde_json::Map<String, serde_json::Value> = serde_json::json!({
            "summary": "Fix login",
            "status": { "name": "In Progress" },
            "assignee": { "displayName": "Neo" },
            "issuetype": { "name": "Bug" },
            "priority": { "name": "High" }
        })
        .as_object()
        .expect("fields object")
        .clone()
        .into_iter()
        .collect();
        let page = generated_types::SearchAndReconcileResults {
            is_last: Some(false),
            next_page_token: Some("next-token".to_owned()),
            issues: vec![generated_types::IssueBean {
                id: Some("10002".to_owned()),
                key: Some("PROJ-1".to_owned()),
                fields,
            }],
        };

        let page = JiraIssueSearchPage::from(page);

        assert_eq!(page.is_last, Some(false));
        assert_eq!(page.next_page_token.as_deref(), Some("next-token"));
        assert_eq!(page.issues[0].summary(), Some("Fix login"));
        assert_eq!(page.issues[0].status_name(), Some("In Progress"));
    }

    #[test]
    fn parses_issue_detail() {
        let issue: JiraIssue = serde_json::from_str(
            r#"{
                "id": "10002",
                "key": "PROJ-1",
                "fields": {
                    "summary": "Fix login",
                    "status": { "name": "Done" },
                    "assignee": null,
                    "issuetype": { "name": "Task" },
                    "priority": { "name": "Medium" }
                }
            }"#,
        )
        .expect("parse issue detail");

        assert_eq!(issue.key.as_deref(), Some("PROJ-1"));
        assert_eq!(issue.summary(), Some("Fix login"));
        assert_eq!(issue.status_name(), Some("Done"));
        assert_eq!(issue.assignee_display_name(), None);
        assert_eq!(issue.issue_type_name(), Some("Task"));
        assert_eq!(issue.priority_name(), Some("Medium"));
    }

    #[test]
    fn parses_comment_page() {
        let page: JiraCommentPage = serde_json::from_str(
            r#"{
                "startAt": 0,
                "maxResults": 25,
                "total": 1,
                "comments": [
                    {
                        "id": "c1",
                        "bodyText": "Hello world",
                        "authorDisplayName": "Alice",
                        "created": "2026-05-01T00:00:00.000Z",
                        "updated": null
                    }
                ]
            }"#,
        )
        .expect("parse comment page");

        assert_eq!(page.start_at, 0);
        assert_eq!(page.max_results, 25);
        assert_eq!(page.total, Some(1));
        assert_eq!(page.comments.len(), 1);
        assert_eq!(page.comments[0].id.as_deref(), Some("c1"));
        assert_eq!(
            page.comments[0].author_display_name.as_deref(),
            Some("Alice")
        );
    }

    #[test]
    fn converts_generated_comment_page() {
        let page = generated_types::PageOfComments {
            start_at: Some(0),
            max_results: Some(25),
            total: Some(1),
            comments: vec![generated_types::Comment {
                id: Some("c1".to_owned()),
                body: adf_body("Hello"),
                author: Some(generated_types::User {
                    account_id: Some("a1".to_owned()),
                    display_name: Some("Alice".to_owned()),
                    active: Some(true),
                }),
                created: Some("2026-05-01T00:00:00.000Z".to_owned()),
                updated: None,
                self_: None,
            }],
        };

        let page = JiraCommentPage::from(page);

        assert_eq!(page.start_at, 0);
        assert_eq!(page.max_results, 25);
        assert_eq!(page.total, Some(1));
        assert_eq!(page.comments.len(), 1);
        assert_eq!(page.comments[0].id.as_deref(), Some("c1"));
        assert_eq!(page.comments[0].body_text.as_deref(), Some("Hello"));
        assert_eq!(
            page.comments[0].author_display_name.as_deref(),
            Some("Alice")
        );
    }

    #[test]
    fn parses_worklog_page() {
        let page: JiraWorklogPage = serde_json::from_str(
            r#"{
                "startAt": 0,
                "maxResults": 20,
                "total": 1,
                "worklogs": [
                    {
                        "id": "w1",
                        "timeSpent": "2h",
                        "timeSpentSeconds": 7200,
                        "author": {
                            "displayName": "Bob",
                            "accountId": "b1"
                        },
                        "started": "2026-06-01T09:00:00.000+0000"
                    }
                ]
            }"#,
        )
        .expect("parse worklog page");

        assert_eq!(page.start_at, 0);
        assert_eq!(page.max_results, 20);
        assert_eq!(page.total, Some(1));
        assert_eq!(page.worklogs.len(), 1);
        let wl = &page.worklogs[0];
        assert_eq!(wl.id.as_deref(), Some("w1"));
        assert_eq!(wl.time_spent.as_deref(), Some("2h"));
        assert_eq!(wl.time_spent_seconds, Some(7200));
        assert_eq!(
            wl.author.as_ref().unwrap().display_name.as_deref(),
            Some("Bob")
        );
        assert_eq!(wl.started.as_deref(), Some("2026-06-01T09:00:00.000+0000"));
    }

    #[test]
    fn worklog_comment_text_extraction() {
        let wl = JiraWorklog {
            id: Some("w1".to_owned()),
            time_spent: Some("1h".to_owned()),
            time_spent_seconds: Some(3600),
            author: None,
            comment: Some(serde_json::json!({
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            { "type": "text", "text": "Reviewed PR" }
                        ]
                    }
                ]
            })),
            started: None,
        };

        assert_eq!(wl.comment_text().as_deref(), Some("Reviewed PR"));
    }

    #[test]
    fn worklog_no_comment_returns_none() {
        let wl = JiraWorklog {
            id: Some("w2".to_owned()),
            time_spent: Some("30m".to_owned()),
            time_spent_seconds: Some(1800),
            author: None,
            comment: None,
            started: None,
        };

        assert_eq!(wl.comment_text(), None);
    }

    #[test]
    fn converts_generated_issue_type() {
        let issue_type = JiraIssueType::from(generated_types::IssueType {
            id: Some("1".to_owned()),
            name: Some("Task".to_owned()),
            description: Some("A task".to_owned()),
            subtask: Some(false),
            ..Default::default()
        });

        assert_eq!(issue_type.id.as_deref(), Some("1"));
        assert_eq!(issue_type.name.as_deref(), Some("Task"));
        assert_eq!(issue_type.description.as_deref(), Some("A task"));
        assert_eq!(issue_type.subtask, Some(false));
    }

    #[test]
    fn parses_issue_type_from_json() {
        let issue_type: JiraIssueType = serde_json::from_value(serde_json::json!({
            "id": "2",
            "name": "Bug",
            "description": "A bug report",
            "subtask": true
        }))
        .expect("parse issue type");

        assert_eq!(issue_type.id.as_deref(), Some("2"));
        assert_eq!(issue_type.name.as_deref(), Some("Bug"));
        assert_eq!(issue_type.description.as_deref(), Some("A bug report"));
        assert_eq!(issue_type.subtask, Some(true));
    }

    #[test]
    fn issue_list_no_other_filters() {
        let list = JiraIssueList {
            project_key: Some("PROJ".to_owned()),
            status: None,
            issue_type: None,
            assignee: None,
            jql: None,
            max_results: 50,
            fields: None,
        };

        let search = list.to_search(None).expect("search");

        assert_eq!(search.jql, "project = \"PROJ\" ORDER BY updated DESC");
    }

    #[test]
    fn issue_list_assignee_literal() {
        let list = JiraIssueList {
            project_key: Some("PROJ".to_owned()),
            status: None,
            issue_type: None,
            assignee: Some("alice".to_owned()),
            jql: None,
            max_results: 50,
            fields: None,
        };

        let search = list.to_search(None).expect("search");

        assert_eq!(
            search.jql,
            "project = \"PROJ\" AND assignee = \"alice\" ORDER BY updated DESC"
        );
    }

    #[test]
    fn issue_list_project_key_overrides_default() {
        let list = JiraIssueList {
            project_key: Some("MINE".to_owned()),
            status: None,
            issue_type: None,
            assignee: None,
            jql: None,
            max_results: 50,
            fields: None,
        };

        let search = list.to_search(Some("OTHER")).expect("search");

        assert_eq!(search.jql, "project = \"MINE\" ORDER BY updated DESC");
    }

    #[test]
    fn issue_list_requires_project() {
        let list = JiraIssueList {
            project_key: None,
            status: None,
            issue_type: None,
            assignee: None,
            jql: None,
            max_results: 50,
            fields: None,
        };

        let error = list.to_search(None).expect_err("should require project");

        assert!(error.to_string().contains("provide --project or --jql"));
    }

    #[test]
    fn issue_list_jql_without_project() {
        let list = JiraIssueList {
            project_key: None,
            status: None,
            issue_type: None,
            assignee: None,
            jql: Some("status = Open".to_owned()),
            max_results: 50,
            fields: None,
        };

        let search = list.to_search(None).expect("search");

        assert_eq!(search.jql, "status = Open");
    }

    #[test]
    fn jql_with_order_by_is_used_verbatim_when_no_explicit_project() {
        let list = JiraIssueList {
            project_key: None,
            status: None,
            issue_type: None,
            assignee: None,
            jql: Some("project = DEMO ORDER BY updated DESC".to_owned()),
            max_results: 10,
            fields: None,
        };

        let search = list.to_search(Some("DEMO")).expect("search");

        assert_eq!(search.jql, "project = DEMO ORDER BY updated DESC");
    }

    #[test]
    fn quote_jql_value_simple() {
        assert_eq!(util::quote_jql_value("PROJ"), "\"PROJ\"");
    }

    #[test]
    fn quote_jql_value_escapes_double_quote() {
        assert_eq!(
            util::quote_jql_value("say \"hello\""),
            "\"say \\\"hello\\\"\""
        );
    }

    #[test]
    fn quote_jql_value_escapes_backslash() {
        assert_eq!(util::quote_jql_value("path\\file"), "\"path\\\\file\"");
    }

    #[test]
    fn parses_single_sprint_detail() {
        let sprint: JiraSprint = serde_json::from_str(
            r#"{
                "id": 42,
                "self": "https://example.atlassian.net/rest/agile/1.0/sprint/42",
                "state": "active",
                "name": "Sprint 5",
                "startDate": "2026-06-01T00:00:00.000Z",
                "endDate": "2026-06-14T00:00:00.000Z",
                "completeDate": null,
                "originBoardId": 10,
                "goal": "Deliver MVP"
            }"#,
        )
        .expect("parse sprint");

        assert_eq!(sprint.id, Some(42));
        assert_eq!(sprint.state.as_deref(), Some("active"));
        assert_eq!(sprint.name.as_deref(), Some("Sprint 5"));
        assert_eq!(
            sprint.start_date.as_deref(),
            Some("2026-06-01T00:00:00.000Z")
        );
        assert_eq!(sprint.end_date.as_deref(), Some("2026-06-14T00:00:00.000Z"));
        assert_eq!(sprint.complete_date, None);
        assert_eq!(sprint.origin_board_id, Some(10));
        assert_eq!(sprint.goal.as_deref(), Some("Deliver MVP"));
    }

    #[test]
    fn parses_sprint_with_missing_optional_fields() {
        let sprint: JiraSprint = serde_json::from_value(serde_json::json!({
            "id": 99,
            "name": "Backlog Sprint"
        }))
        .expect("parse sprint with minimal fields");

        assert_eq!(sprint.id, Some(99));
        assert_eq!(sprint.name.as_deref(), Some("Backlog Sprint"));
        assert_eq!(sprint.state, None);
        assert_eq!(sprint.start_date, None);
        assert_eq!(sprint.end_date, None);
        assert_eq!(sprint.origin_board_id, None);
        assert_eq!(sprint.goal, None);
    }

    #[test]
    fn parses_user_from_json() {
        let user: JiraUser = serde_json::from_value(serde_json::json!({
            "accountId": "abc-123",
            "displayName": "Charlie",
            "active": true
        }))
        .expect("parse user");

        assert_eq!(user.account_id.as_deref(), Some("abc-123"));
        assert_eq!(user.display_name.as_deref(), Some("Charlie"));
        assert_eq!(user.active, Some(true));
    }

    #[test]
    fn parses_empty_issue_search_page() {
        let page: JiraIssueSearchPage = serde_json::from_value(serde_json::json!({
            "isLast": true,
            "issues": []
        }))
        .expect("parse empty issue search page");

        assert_eq!(page.is_last, Some(true));
        assert!(page.issues.is_empty());
    }

    #[test]
    fn parses_empty_sprint_page() {
        let page: JiraSprintPage = serde_json::from_value(serde_json::json!({
            "startAt": 0,
            "maxResults": 50,
            "total": 0,
            "isLast": true,
            "values": []
        }))
        .expect("parse empty sprint page");

        assert_eq!(page.total, Some(0));
        assert!(page.values.is_empty());
    }

    #[test]
    fn parses_empty_comment_page() {
        let page: JiraCommentPage = serde_json::from_value(serde_json::json!({
            "startAt": 0,
            "maxResults": 25,
            "total": 0,
            "comments": []
        }))
        .expect("parse empty comment page");

        assert_eq!(page.total, Some(0));
        assert!(page.comments.is_empty());
    }
}
