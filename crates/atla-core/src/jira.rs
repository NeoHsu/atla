use atla_jira_api::{apis as generated_apis, models as generated_models};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::client::{ApiError, AtlassianClient, read_empty, read_json};

#[derive(Debug, Clone)]
pub struct JiraClient {
    raw_client: AtlassianClient,
    generated: generated_apis::configuration::Configuration,
}

impl JiraClient {
    pub fn new(client: AtlassianClient) -> Self {
        let generated = generated_apis::configuration::Configuration {
            base_path: client.instance().base_url.clone(),
            user_agent: Some("atla".to_owned()),
            basic_auth: Some((client.email().to_owned(), Some(client.token().to_owned()))),
            ..Default::default()
        };

        Self {
            raw_client: client,
            generated,
        }
    }

    pub fn instance_url(&self) -> &str {
        &self.raw_client.instance().base_url
    }

    pub async fn search_projects(
        &self,
        search: &JiraProjectSearch,
    ) -> Result<JiraProjectPage, ApiError> {
        let page = generated_apis::projects_api::search_projects(
            &self.generated,
            Some(search.start_at.min(i64::MAX as u64) as i64),
            Some(limit_i32(search.max_results)),
            search.query.as_deref(),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn get_project(&self, project_id_or_key: &str) -> Result<JiraProject, ApiError> {
        generated_apis::projects_api::get_project(&self.generated, project_id_or_key)
            .await
            .map(JiraProject::from)
            .map_err(generated_error)
    }

    pub async fn list_issue_types(
        &self,
        project_id_or_key: &str,
    ) -> Result<Vec<JiraIssueType>, ApiError> {
        let project = self.get_project(project_id_or_key).await?;
        let project_id = project.id.ok_or_else(|| {
            ApiError::Decode(format!(
                "project `{project_id_or_key}` did not include an id"
            ))
        })?;

        read_json(
            self.raw_client
                .get("/rest/api/3/issuetype/project")
                .query(&[("projectId", project_id)]),
        )
        .await
    }

    pub async fn create_issue(
        &self,
        issue: &JiraIssueCreate,
    ) -> Result<JiraCreatedIssue, ApiError> {
        generated_apis::issues_api::create_issue(&self.generated, issue.to_generated())
            .await
            .map(JiraCreatedIssue::from)
            .map_err(generated_error)
    }

    pub async fn update_issue(&self, issue: &JiraIssueUpdate) -> Result<(), ApiError> {
        generated_apis::issues_api::edit_issue(
            &self.generated,
            &issue.issue_id_or_key,
            issue.to_generated(),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn update_issue_labels(&self, labels: &JiraIssueLabelUpdate) -> Result<(), ApiError> {
        read_empty(
            self.raw_client
                .put(&format!("/rest/api/3/issue/{}", labels.issue_id_or_key))
                .json(&labels.to_json()),
        )
        .await
    }

    pub async fn search_issues(
        &self,
        search: &JiraIssueSearch,
    ) -> Result<JiraIssueSearchPage, ApiError> {
        let page = generated_apis::issue_search_api::search_and_reconsile_issues_using_jql(
            &self.generated,
            Some(&search.jql),
            None,
            Some(limit_i32(search.max_results)),
            Some(search.issue_fields()),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn get_issue(
        &self,
        issue_id_or_key: &str,
        fields: Option<Vec<String>>,
    ) -> Result<JiraIssue, ApiError> {
        generated_apis::issues_api::get_issue(
            &self.generated,
            issue_id_or_key,
            Some(issue_fields(fields.as_deref())),
        )
        .await
        .map(JiraIssue::from)
        .map_err(generated_error)
    }

    pub async fn list_transitions(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraTransition>, ApiError> {
        let transitions: JiraTransitionPage = read_json(
            self.raw_client
                .get(&format!("/rest/api/3/issue/{issue_id_or_key}/transitions"))
                .query(&[("expand", "transitions.fields")]),
        )
        .await?;

        Ok(transitions.transitions.into_iter().collect())
    }

    pub async fn transition_issue(
        &self,
        issue_id_or_key: &str,
        transition_id_or_name: &str,
        fields: serde_json::Map<String, serde_json::Value>,
    ) -> Result<JiraTransition, ApiError> {
        let transitions = self.list_transitions(issue_id_or_key).await?;
        let transition = transitions
            .iter()
            .find(|transition| {
                transition.id.as_deref() == Some(transition_id_or_name)
                    || transition
                        .name
                        .as_deref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(transition_id_or_name))
            })
            .cloned()
            .ok_or_else(|| {
                let available = transitions
                    .iter()
                    .filter_map(|transition| transition.name.as_deref())
                    .collect::<Vec<_>>()
                    .join(", ");
                if available.is_empty() {
                    ApiError::Decode(format!(
                        "transition `{transition_id_or_name}` not available for issue `{issue_id_or_key}`"
                    ))
                } else {
                    ApiError::Decode(format!(
                        "transition `{transition_id_or_name}` not available for issue `{issue_id_or_key}`; available: {available}"
                    ))
                }
            })?;
        let transition_id = transition.id.clone().ok_or_else(|| {
            ApiError::Decode("selected transition did not include an id".to_owned())
        })?;
        let missing_fields = transition
            .required_fields()
            .into_iter()
            .filter(|field| !fields.contains_key(*field))
            .collect::<Vec<_>>();
        if !missing_fields.is_empty() {
            return Err(ApiError::Decode(format!(
                "transition `{transition_id_or_name}` requires field(s): {}",
                missing_fields.join(", ")
            )));
        }

        let mut request = serde_json::Map::new();
        request.insert(
            "transition".to_owned(),
            serde_json::json!({ "id": transition_id }),
        );
        if !fields.is_empty() {
            request.insert("fields".to_owned(), serde_json::Value::Object(fields));
        }
        read_empty(
            self.raw_client
                .post(&format!("/rest/api/3/issue/{issue_id_or_key}/transitions"))
                .json(&serde_json::Value::Object(request)),
        )
        .await?;

        Ok(transition)
    }

    pub async fn add_comment(
        &self,
        issue_id_or_key: &str,
        body: &str,
    ) -> Result<JiraComment, ApiError> {
        let comment = generated_apis::issue_comments_api::add_comment(
            &self.generated,
            issue_id_or_key,
            generated_models::CommentCreateRequest::new(adf_body(body)),
        )
        .await
        .map_err(generated_error)?;

        Ok(comment.into())
    }

    pub async fn update_comment(
        &self,
        issue_id_or_key: &str,
        comment_id: &str,
        body: &str,
    ) -> Result<JiraComment, ApiError> {
        let value: serde_json::Value = read_json(
            self.raw_client
                .put(&format!(
                    "/rest/api/3/issue/{issue_id_or_key}/comment/{comment_id}"
                ))
                .json(&serde_json::json!({ "body": text_adf(body) })),
        )
        .await?;

        jira_comment_from_value(value)
    }

    pub async fn list_comments(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
    ) -> Result<JiraCommentPage, ApiError> {
        let page = generated_apis::issue_comments_api::get_comments(
            &self.generated,
            issue_id_or_key,
            Some(0),
            Some(limit_i32(max_results)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn delete_comment(
        &self,
        issue_id_or_key: &str,
        comment_id: &str,
    ) -> Result<(), ApiError> {
        read_empty(self.raw_client.delete(&format!(
            "/rest/api/3/issue/{issue_id_or_key}/comment/{comment_id}"
        )))
        .await
    }

    pub async fn create_issue_link(&self, link: &JiraIssueLinkCreate) -> Result<(), ApiError> {
        read_empty(
            self.raw_client
                .post("/rest/api/3/issueLink")
                .json(&link.to_json()),
        )
        .await
    }

    pub async fn list_issue_links(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraIssueLink>, ApiError> {
        let value: serde_json::Value = read_json(
            self.raw_client
                .get(&format!("/rest/api/3/issue/{issue_id_or_key}"))
                .query(&[("fields", "issuelinks")]),
        )
        .await?;

        Ok(value
            .get("fields")
            .and_then(|fields| fields.get("issuelinks"))
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .map(jira_issue_link_from_value)
            .collect())
    }

    pub async fn get_attachment(&self, id: &str) -> Result<JiraAttachment, ApiError> {
        read_json(self.raw_client.get(&format!("/rest/api/3/attachment/{id}"))).await
    }

    pub async fn list_issue_attachments(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraAttachment>, ApiError> {
        let issue = self
            .get_issue(issue_id_or_key, Some(vec!["attachment".to_owned()]))
            .await?;
        issue
            .fields
            .get("attachment")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| ApiError::Decode(format!("failed to decode attachments: {error}")))
    }

    pub async fn download_attachment(
        &self,
        id: &str,
        output: Option<&Path>,
    ) -> Result<JiraAttachmentDownload, ApiError> {
        let attachment = self.get_attachment(id).await?;
        self.download_attachment_metadata(attachment, output).await
    }

    pub async fn download_issue_attachments(
        &self,
        issue_id_or_key: &str,
        output_dir: Option<&Path>,
    ) -> Result<Vec<JiraAttachmentDownload>, ApiError> {
        let attachments = self.list_issue_attachments(issue_id_or_key).await?;
        let mut downloads = Vec::new();
        for attachment in attachments {
            let output = output_dir.map(|dir| dir.join(attachment_filename(&attachment)));
            downloads.push(
                self.download_attachment_metadata(attachment, output.as_deref())
                    .await?,
            );
        }

        Ok(downloads)
    }

    async fn download_attachment_metadata(
        &self,
        attachment: JiraAttachment,
        output: Option<&Path>,
    ) -> Result<JiraAttachmentDownload, ApiError> {
        let id = attachment.id.as_deref().unwrap_or("attachment");
        let content = attachment.content.clone().ok_or_else(|| {
            ApiError::Decode(format!("attachment `{id}` did not include a content URL"))
        })?;
        let response = self.raw_client.get(&content).send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, body });
        }
        let bytes = response.bytes().await.map_err(ApiError::Request)?;
        let filename = attachment_filename(&attachment);
        let path = attachment_output_path(output, filename);
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|error| {
                ApiError::Decode(format!("failed to create {}: {error}", parent.display()))
            })?;
        }
        std::fs::write(&path, &bytes).map_err(|error| {
            ApiError::Decode(format!("failed to write {}: {error}", path.display()))
        })?;

        Ok(JiraAttachmentDownload {
            attachment,
            path,
            bytes: bytes.len() as u64,
        })
    }

    pub async fn delete_issue_link(&self, link_id: &str) -> Result<(), ApiError> {
        read_empty(
            self.raw_client
                .delete(&format!("/rest/api/3/issueLink/{link_id}")),
        )
        .await
    }

    pub async fn delete_issue(
        &self,
        issue_id_or_key: &str,
        delete_subtasks: bool,
    ) -> Result<(), ApiError> {
        read_empty(
            self.raw_client
                .delete(&format!("/rest/api/3/issue/{issue_id_or_key}"))
                .query(&[("deleteSubtasks", delete_subtasks)]),
        )
        .await
    }

    pub async fn assign_issue(&self, assign: &JiraIssueAssign) -> Result<JiraUser, ApiError> {
        let user = match &assign.target {
            JiraAssigneeTarget::Me => self.current_user().await?,
            JiraAssigneeTarget::AccountId(account_id) => JiraUser {
                account_id: Some(account_id.clone()),
                display_name: None,
                active: None,
            },
            JiraAssigneeTarget::Query(query) => {
                let users = self
                    .find_assignable_users(&assign.issue_id_or_key, query)
                    .await?;
                resolve_assignable_user(query, users)?
            }
        };
        let account_id = user.account_id.clone().ok_or_else(|| {
            ApiError::Decode("selected Jira user did not include an accountId".to_owned())
        })?;

        read_empty(
            self.raw_client
                .put(&format!(
                    "/rest/api/3/issue/{}/assignee",
                    assign.issue_id_or_key
                ))
                .json(&JiraAssignIssueRequest { account_id }),
        )
        .await?;

        Ok(user)
    }

    pub async fn search_boards(&self, search: &JiraBoardSearch) -> Result<JiraBoardPage, ApiError> {
        let mut query = vec![
            ("startAt", search.start_at.to_string()),
            ("maxResults", search.max_results.to_string()),
        ];
        if let Some(board_type) = &search.board_type {
            query.push(("type", board_type.clone()));
        }
        if let Some(name) = &search.name {
            query.push(("name", name.clone()));
        }
        if let Some(project_key_or_id) = &search.project_key_or_id {
            query.push(("projectKeyOrId", project_key_or_id.clone()));
        }

        read_json(self.raw_client.get("/rest/agile/1.0/board").query(&query)).await
    }

    pub async fn list_sprints(
        &self,
        search: &JiraSprintSearch,
    ) -> Result<JiraSprintPage, ApiError> {
        let mut query = vec![
            ("startAt", search.start_at.to_string()),
            ("maxResults", search.max_results.to_string()),
        ];
        if let Some(state) = &search.state {
            query.push(("state", state.clone()));
        }

        read_json(
            self.raw_client
                .get(&format!("/rest/agile/1.0/board/{}/sprint", search.board_id))
                .query(&query),
        )
        .await
    }

    pub async fn get_sprint(&self, sprint_id: u64) -> Result<JiraSprint, ApiError> {
        read_json(
            self.raw_client
                .get(&format!("/rest/agile/1.0/sprint/{sprint_id}")),
        )
        .await
    }

    pub async fn create_sprint(&self, sprint: &JiraSprintCreate) -> Result<JiraSprint, ApiError> {
        read_json(
            self.raw_client
                .post("/rest/agile/1.0/sprint")
                .json(&sprint.to_json()),
        )
        .await
    }

    pub async fn update_sprint(&self, update: &JiraSprintUpdate) -> Result<JiraSprint, ApiError> {
        read_json(
            self.raw_client
                .put(&format!("/rest/agile/1.0/sprint/{}", update.id))
                .json(&update.to_json()),
        )
        .await
    }

    pub async fn move_issues_to_sprint(
        &self,
        sprint_id: u64,
        issues: &[String],
    ) -> Result<(), ApiError> {
        read_empty(
            self.raw_client
                .post(&format!("/rest/agile/1.0/sprint/{sprint_id}/issue"))
                .json(&serde_json::json!({ "issues": issues })),
        )
        .await
    }

    pub async fn move_issues_to_backlog(&self, issues: &[String]) -> Result<(), ApiError> {
        read_empty(
            self.raw_client
                .post("/rest/agile/1.0/backlog/issue")
                .json(&serde_json::json!({ "issues": issues })),
        )
        .await
    }

    pub async fn add_worklog(&self, worklog: &JiraWorklogCreate) -> Result<JiraWorklog, ApiError> {
        read_json(
            self.raw_client
                .post(&format!(
                    "/rest/api/3/issue/{}/worklog",
                    worklog.issue_id_or_key
                ))
                .json(&worklog.to_json()),
        )
        .await
    }

    pub async fn list_worklogs(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
    ) -> Result<JiraWorklogPage, ApiError> {
        read_json(
            self.raw_client
                .get(&format!("/rest/api/3/issue/{issue_id_or_key}/worklog"))
                .query(&[
                    ("startAt", "0".to_owned()),
                    ("maxResults", limit_i32(max_results).to_string()),
                ]),
        )
        .await
    }

    async fn current_user(&self) -> Result<JiraUser, ApiError> {
        read_json(self.raw_client.get("/rest/api/3/myself")).await
    }

    async fn find_assignable_users(
        &self,
        issue_id_or_key: &str,
        query_text: &str,
    ) -> Result<Vec<JiraUser>, ApiError> {
        read_json(
            self.raw_client
                .get("/rest/api/3/user/assignable/search")
                .query(&[
                    ("issueKey", issue_id_or_key.to_owned()),
                    ("query", query_text.to_owned()),
                    ("maxResults", "50".to_owned()),
                ]),
        )
        .await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraProjectSearch {
    pub start_at: u64,
    pub max_results: u32,
    pub query: Option<String>,
}

impl Default for JiraProjectSearch {
    fn default() -> Self {
        Self {
            start_at: 0,
            max_results: 50,
            query: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueSearch {
    pub jql: String,
    pub max_results: u32,
    pub fields: Option<Vec<String>>,
}

impl JiraIssueSearch {
    fn issue_fields(&self) -> Vec<String> {
        issue_fields(self.fields.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueList {
    pub project_key: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub jql: Option<String>,
    pub max_results: u32,
    pub fields: Option<Vec<String>>,
}

impl JiraIssueList {
    pub fn to_search(&self, default_project: Option<&str>) -> Result<JiraIssueSearch, ApiError> {
        if let Some(jql) = &self.jql {
            let final_jql = if let Some(project) = self.project_key.as_deref().or(default_project)
            {
                format!("project = {} AND ({jql})", quote_jql_value(project))
            } else {
                jql.clone()
            };
            return Ok(JiraIssueSearch {
                jql: final_jql,
                max_results: self.max_results,
                fields: self.fields.clone(),
            });
        }

        let project_key = self
            .project_key
            .as_deref()
            .or(default_project)
            .ok_or_else(|| {
                ApiError::Decode(
                    "provide --project or --jql, or set config default-project".to_owned(),
                )
            })?;
        let mut clauses = vec![format!("project = {}", quote_jql_value(project_key))];

        if let Some(status) = &self.status {
            clauses.push(format!("status = {}", quote_jql_value(status)));
        }
        if let Some(assignee) = &self.assignee {
            if assignee.eq_ignore_ascii_case("me") {
                clauses.push("assignee = currentUser()".to_owned());
            } else {
                clauses.push(format!("assignee = {}", quote_jql_value(assignee)));
            }
        }

        Ok(JiraIssueSearch {
            jql: format!("{} ORDER BY updated DESC", clauses.join(" AND ")),
            max_results: self.max_results,
            fields: self.fields.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueAssign {
    pub issue_id_or_key: String,
    pub target: JiraAssigneeTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JiraAssigneeTarget {
    Me,
    AccountId(String),
    Query(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraCreatedIssue {
    pub id: Option<String>,
    pub key: Option<String>,
    pub self_url: Option<String>,
}

impl From<generated_models::CreatedIssue> for JiraCreatedIssue {
    fn from(issue: generated_models::CreatedIssue) -> Self {
        Self {
            id: issue.id,
            key: issue.key,
            self_url: issue.param_self,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JiraIssueCreate {
    pub project_key: String,
    pub issue_type: String,
    pub summary: String,
    pub description: Option<String>,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl JiraIssueCreate {
    fn to_generated(&self) -> generated_models::IssueUpdateDetails {
        let mut fields = self.fields.clone();
        fields.insert(
            "project".to_owned(),
            serde_json::json!({ "key": self.project_key }),
        );
        fields.insert(
            "issuetype".to_owned(),
            serde_json::json!({ "name": self.issue_type }),
        );
        fields.insert("summary".to_owned(), self.summary.clone().into());
        if let Some(description) = &self.description {
            fields.insert("description".to_owned(), text_adf(description));
        }

        generated_models::IssueUpdateDetails {
            fields: Some(fields.into_iter().collect()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JiraIssueUpdate {
    pub issue_id_or_key: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueLabelUpdate {
    pub issue_id_or_key: String,
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

impl JiraIssueLabelUpdate {
    pub fn is_empty(&self) -> bool {
        self.add.is_empty() && self.remove.is_empty()
    }

    fn to_json(&self) -> serde_json::Value {
        let mut operations = Vec::new();
        for label in &self.add {
            operations.push(serde_json::json!({ "add": label }));
        }
        for label in &self.remove {
            operations.push(serde_json::json!({ "remove": label }));
        }

        serde_json::json!({
            "update": {
                "labels": operations
            }
        })
    }
}

impl JiraIssueUpdate {
    pub fn is_empty(&self) -> bool {
        self.summary.is_none() && self.description.is_none() && self.fields.is_empty()
    }

    fn to_generated(&self) -> generated_models::IssueUpdateDetails {
        let mut fields = self.fields.clone();
        if let Some(summary) = &self.summary {
            fields.insert("summary".to_owned(), summary.clone().into());
        }
        if let Some(description) = &self.description {
            fields.insert("description".to_owned(), text_adf(description));
        }

        generated_models::IssueUpdateDetails {
            fields: Some(fields.into_iter().collect()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueSearchPage {
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub issues: Vec<JiraIssue>,
}

impl From<generated_models::SearchAndReconcileResults> for JiraIssueSearchPage {
    fn from(page: generated_models::SearchAndReconcileResults) -> Self {
        Self {
            is_last: page.is_last,
            next_page_token: page.next_page_token,
            issues: page
                .issues
                .unwrap_or_default()
                .into_iter()
                .map(JiraIssue::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssue {
    pub id: Option<String>,
    pub key: Option<String>,
    #[serde(default)]
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl JiraIssue {
    pub fn summary(&self) -> Option<&str> {
        self.field_string("summary")
    }

    pub fn status_name(&self) -> Option<&str> {
        self.field_object_string("status", "name")
    }

    pub fn assignee_display_name(&self) -> Option<&str> {
        self.field_object_string("assignee", "displayName")
    }

    pub fn issue_type_name(&self) -> Option<&str> {
        self.field_object_string("issuetype", "name")
    }

    pub fn priority_name(&self) -> Option<&str> {
        self.field_object_string("priority", "name")
    }

    fn field_string(&self, name: &str) -> Option<&str> {
        self.fields.get(name)?.as_str()
    }

    fn field_object_string(&self, object: &str, name: &str) -> Option<&str> {
        self.fields.get(object)?.get(name)?.as_str()
    }
}

impl From<generated_models::IssueBean> for JiraIssue {
    fn from(issue: generated_models::IssueBean) -> Self {
        Self {
            id: issue.id,
            key: issue.key,
            fields: issue
                .fields
                .unwrap_or_default()
                .into_iter()
                .collect::<serde_json::Map<String, serde_json::Value>>(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraTransition {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "toStatus", alias = "to")]
    pub to_status: Option<JiraStatus>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl JiraTransition {
    pub fn required_fields(&self) -> Vec<&str> {
        self.fields
            .iter()
            .filter_map(|(id, metadata)| {
                metadata
                    .get("required")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                    .then_some(id.as_str())
            })
            .collect()
    }
}

impl From<generated_models::Transition> for JiraTransition {
    fn from(transition: generated_models::Transition) -> Self {
        Self {
            id: transition.id,
            name: transition.name,
            to_status: transition.to.map(|status| JiraStatus::from(*status)),
            fields: serde_json::Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JiraTransitionPage {
    #[serde(default)]
    transitions: Vec<JiraTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraStatus {
    pub id: Option<String>,
    pub name: Option<String>,
}

impl From<generated_models::Status> for JiraStatus {
    fn from(status: generated_models::Status) -> Self {
        Self {
            id: status.id,
            name: status.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraCommentPage {
    #[serde(default)]
    pub start_at: u32,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u32>,
    #[serde(default)]
    pub comments: Vec<JiraComment>,
}

impl From<generated_models::PageOfComments> for JiraCommentPage {
    fn from(page: generated_models::PageOfComments) -> Self {
        Self {
            start_at: page.start_at.unwrap_or_default().max(0) as u32,
            max_results: page.max_results.unwrap_or_default().max(0) as u32,
            total: page.total.map(|total| total.max(0) as u32),
            comments: page
                .comments
                .unwrap_or_default()
                .into_iter()
                .map(JiraComment::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComment {
    pub id: Option<String>,
    pub body_text: Option<String>,
    pub author_display_name: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

impl From<generated_models::Comment> for JiraComment {
    fn from(comment: generated_models::Comment) -> Self {
        Self {
            id: comment.id,
            body_text: comment
                .body
                .as_ref()
                .map(adf_plain_text)
                .filter(|text| !text.is_empty()),
            author_display_name: comment.author.and_then(|author| author.display_name),
            created: comment.created,
            updated: comment.updated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueLinkCreate {
    pub source_key: String,
    pub target_key: String,
    pub link_type: String,
}

impl JiraIssueLinkCreate {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": { "name": self.link_type },
            "inwardIssue": { "key": self.source_key },
            "outwardIssue": { "key": self.target_key }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueLink {
    pub id: Option<String>,
    pub link_type: Option<String>,
    pub inward_issue: Option<JiraLinkedIssue>,
    pub outward_issue: Option<JiraLinkedIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraAttachment {
    pub id: Option<String>,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
    pub author: Option<JiraUser>,
    pub created: Option<String>,
    pub content: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraAttachmentDownload {
    pub attachment: JiraAttachment,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraLinkedIssue {
    pub id: Option<String>,
    pub key: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraUser {
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JiraAssignIssueRequest {
    account_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraBoardSearch {
    pub start_at: u64,
    pub max_results: u32,
    pub board_type: Option<String>,
    pub name: Option<String>,
    pub project_key_or_id: Option<String>,
}

impl Default for JiraBoardSearch {
    fn default() -> Self {
        Self {
            start_at: 0,
            max_results: 50,
            board_type: None,
            name: None,
            project_key_or_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraBoardPage {
    #[serde(default)]
    pub start_at: u64,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub values: Vec<JiraBoard>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraBoard {
    pub id: Option<u64>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub board_type: Option<String>,
    #[serde(rename = "self")]
    pub self_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraSprintSearch {
    pub board_id: u64,
    pub start_at: u64,
    pub max_results: u32,
    pub state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSprintPage {
    #[serde(default)]
    pub start_at: u64,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub values: Vec<JiraSprint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSprint {
    pub id: Option<u64>,
    #[serde(rename = "self")]
    pub self_url: Option<String>,
    pub state: Option<String>,
    pub name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub complete_date: Option<String>,
    pub origin_board_id: Option<u64>,
    pub goal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraSprintCreate {
    pub board_id: u64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub goal: Option<String>,
}

impl JiraSprintCreate {
    fn to_json(&self) -> serde_json::Value {
        let mut value = serde_json::json!({
            "originBoardId": self.board_id,
            "name": self.name,
        });
        let object = value.as_object_mut().expect("sprint create is object");
        if let Some(start_date) = &self.start_date {
            object.insert("startDate".to_owned(), start_date.clone().into());
        }
        if let Some(end_date) = &self.end_date {
            object.insert("endDate".to_owned(), end_date.clone().into());
        }
        if let Some(goal) = &self.goal {
            object.insert("goal".to_owned(), goal.clone().into());
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraSprintUpdate {
    pub id: u64,
    pub state: Option<String>,
    pub name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub goal: Option<String>,
}

impl JiraSprintUpdate {
    fn to_json(&self) -> serde_json::Value {
        let mut object = serde_json::Map::new();
        if let Some(state) = &self.state {
            object.insert("state".to_owned(), state.clone().into());
        }
        if let Some(name) = &self.name {
            object.insert("name".to_owned(), name.clone().into());
        }
        if let Some(start_date) = &self.start_date {
            object.insert("startDate".to_owned(), start_date.clone().into());
        }
        if let Some(end_date) = &self.end_date {
            object.insert("endDate".to_owned(), end_date.clone().into());
        }
        if let Some(goal) = &self.goal {
            object.insert("goal".to_owned(), goal.clone().into());
        }
        serde_json::Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraWorklogCreate {
    pub issue_id_or_key: String,
    pub time_spent: String,
    pub comment: Option<String>,
}

impl JiraWorklogCreate {
    fn to_json(&self) -> serde_json::Value {
        let mut value = serde_json::json!({ "timeSpent": self.time_spent });
        if let Some(comment) = &self.comment {
            value
                .as_object_mut()
                .expect("worklog create is object")
                .insert("comment".to_owned(), text_adf(comment));
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraWorklogPage {
    #[serde(default)]
    pub start_at: u32,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u32>,
    #[serde(default)]
    pub worklogs: Vec<JiraWorklog>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraWorklog {
    pub id: Option<String>,
    pub time_spent: Option<String>,
    pub time_spent_seconds: Option<u64>,
    pub author: Option<JiraWorklogAuthor>,
    pub comment: Option<serde_json::Value>,
    pub started: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraWorklogAuthor {
    pub display_name: Option<String>,
    pub account_id: Option<String>,
}

impl JiraWorklog {
    pub fn comment_text(&self) -> Option<String> {
        let object = self.comment.as_ref()?.as_object()?;
        Some(adf_plain_text(&object.clone().into_iter().collect())).filter(|text| !text.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueType {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub subtask: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraProjectPage {
    #[serde(default)]
    pub start_at: u64,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub values: Vec<JiraProject>,
}

impl From<generated_models::PageBeanProject> for JiraProjectPage {
    fn from(page: generated_models::PageBeanProject) -> Self {
        Self {
            start_at: page.start_at.unwrap_or_default().max(0) as u64,
            max_results: page.max_results.unwrap_or_default().max(0) as u32,
            total: page.total.map(|total| total.max(0) as u64),
            is_last: page.is_last,
            values: page
                .values
                .unwrap_or_default()
                .into_iter()
                .map(JiraProject::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraProject {
    pub id: Option<String>,
    pub key: Option<String>,
    pub name: Option<String>,
    pub project_type_key: Option<String>,
    pub style: Option<String>,
    pub simplified: Option<bool>,
    pub archived: Option<bool>,
}

impl From<generated_models::Project> for JiraProject {
    fn from(project: generated_models::Project) -> Self {
        Self {
            id: project.id,
            key: project.key,
            name: project.name,
            project_type_key: project.project_type_key.and_then(serialized_string),
            style: project.style.and_then(serialized_string),
            simplified: project.simplified,
            archived: project.archived,
        }
    }
}

pub fn default_issue_fields() -> Vec<String> {
    ["summary", "status", "assignee", "issuetype", "priority"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn issue_fields(fields: Option<&[String]>) -> Vec<String> {
    fields
        .filter(|fields| !fields.is_empty())
        .map(|fields| fields.to_vec())
        .unwrap_or_else(default_issue_fields)
}

fn attachment_output_path(output: Option<&Path>, filename: &str) -> PathBuf {
    match output {
        Some(output) if output.is_dir() => output.join(filename),
        Some(output) => output.to_path_buf(),
        None => PathBuf::from(filename),
    }
}

fn attachment_filename(attachment: &JiraAttachment) -> &str {
    attachment
        .filename
        .as_deref()
        .and_then(|filename| Path::new(filename).file_name()?.to_str())
        .or(attachment.id.as_deref())
        .unwrap_or("attachment")
}

fn limit_i32(limit: u32) -> i32 {
    limit.min(i32::MAX as u32) as i32
}

fn serialized_string<T: Serialize>(value: T) -> Option<String> {
    match serde_json::to_value(value).ok()? {
        serde_json::Value::String(value) => Some(value),
        _ => None,
    }
}

fn text_adf(text: &str) -> serde_json::Value {
    let content = text
        .lines()
        .map(|line| {
            if line.is_empty() {
                serde_json::json!({
                    "type": "paragraph"
                })
            } else {
                serde_json::json!({
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": line
                        }
                    ]
                })
            }
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": content,
    })
}

fn adf_body(text: &str) -> std::collections::HashMap<String, serde_json::Value> {
    text_adf(text)
        .as_object()
        .expect("ADF root is an object")
        .clone()
        .into_iter()
        .collect()
}

fn adf_plain_text(body: &std::collections::HashMap<String, serde_json::Value>) -> String {
    let value = serde_json::Value::Object(body.clone().into_iter().collect());
    let mut parts = Vec::new();
    collect_adf_text(&value, &mut parts);
    parts.join("\n")
}

fn jira_comment_from_value(value: serde_json::Value) -> Result<JiraComment, ApiError> {
    let object = value
        .as_object()
        .ok_or_else(|| ApiError::Decode("Jira comment response was not an object".to_owned()))?;
    let body_text = object
        .get("body")
        .and_then(serde_json::Value::as_object)
        .map(|body| adf_plain_text(&body.clone().into_iter().collect()))
        .filter(|text| !text.is_empty());

    Ok(JiraComment {
        id: object
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        body_text,
        author_display_name: object
            .get("author")
            .and_then(|author| author.get("displayName"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        created: object
            .get("created")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        updated: object
            .get("updated")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
    })
}

fn jira_issue_link_from_value(value: &serde_json::Value) -> JiraIssueLink {
    JiraIssueLink {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        link_type: value
            .get("type")
            .and_then(|link_type| link_type.get("name"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        inward_issue: value.get("inwardIssue").map(jira_linked_issue_from_value),
        outward_issue: value.get("outwardIssue").map(jira_linked_issue_from_value),
    }
}

fn jira_linked_issue_from_value(value: &serde_json::Value) -> JiraLinkedIssue {
    JiraLinkedIssue {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        key: value
            .get("key")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        summary: value
            .get("fields")
            .and_then(|fields| fields.get("summary"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        status: value
            .get("fields")
            .and_then(|fields| fields.get("status"))
            .and_then(|status| status.get("name"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
    }
}

fn collect_adf_text(value: &serde_json::Value, parts: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(serde_json::Value::as_str) {
                parts.push(text.to_owned());
            }
            if let Some(content) = object.get("content") {
                collect_adf_text(content, parts);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_adf_text(item, parts);
            }
        }
        _ => {}
    }
}

fn quote_jql_value(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn resolve_assignable_user(query: &str, users: Vec<JiraUser>) -> Result<JiraUser, ApiError> {
    let exact_matches = users
        .iter()
        .filter(|user| {
            user.account_id.as_deref() == Some(query)
                || user
                    .display_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(query))
        })
        .cloned()
        .collect::<Vec<_>>();
    if exact_matches.len() == 1 {
        return Ok(exact_matches.into_iter().next().expect("one exact match"));
    }

    match users.as_slice() {
        [user] => Ok(user.clone()),
        [] => Err(ApiError::Decode(format!(
            "no assignable Jira user matched `{query}`"
        ))),
        _ => {
            let names = users
                .iter()
                .filter_map(|user| user.display_name.as_deref())
                .collect::<Vec<_>>()
                .join(", ");
            Err(ApiError::Decode(format!(
                "multiple assignable Jira users matched `{query}`; pass --account-id. matches: {names}"
            )))
        }
    }
}

fn generated_error<T>(error: generated_apis::Error<T>) -> ApiError {
    match error {
        generated_apis::Error::Reqwest(error) => ApiError::Decode(error.to_string()),
        generated_apis::Error::Serde(error) => ApiError::Decode(error.to_string()),
        generated_apis::Error::Io(error) => ApiError::Decode(error.to_string()),
        generated_apis::Error::ResponseError(response) => ApiError::Http {
            status: response.status,
            body: response.content,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let page = generated_models::PageBeanProject {
            start_at: Some(0),
            max_results: Some(50),
            total: Some(1),
            is_last: Some(true),
            values: Some(vec![generated_models::Project {
                id: Some("10000".to_owned()),
                key: Some("PROJ".to_owned()),
                name: Some("Project".to_owned()),
                project_type_key: Some(generated_models::project::ProjectTypeKey::Software),
                style: Some(generated_models::project::Style::Classic),
                simplified: Some(false),
                archived: Some(false),
            }]),
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
        let fields = generated.fields.expect("fields");

        assert_eq!(fields["project"], serde_json::json!({ "key": "PROJ" }));
        assert_eq!(fields["issuetype"], serde_json::json!({ "name": "Task" }));
        assert_eq!(fields["summary"], serde_json::json!("Fix login"));
        assert_eq!(fields["priority"], serde_json::json!({ "name": "High" }));
        assert_eq!(fields["description"]["type"], serde_json::json!("doc"));
        assert_eq!(
            fields["description"]["content"].as_array().unwrap().len(),
            2
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
        let fields = generated.fields.expect("fields");

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
            assignee: None,
            jql: Some("status = Open".to_owned()),
            max_results: 10,
            fields: None,
        };

        let search = list.to_search(None).expect("search");

        assert_eq!(
            search.jql,
            "project = \"PROJ\" AND (status = Open)"
        );
        assert_eq!(search.max_results, 10);
    }

    #[test]
    fn issue_search_uses_requested_fields() {
        let search = JiraIssueSearch {
            jql: "project = PROJ".to_owned(),
            max_results: 10,
            fields: Some(vec!["summary".to_owned(), "attachment".to_owned()]),
        };

        assert_eq!(
            search.issue_fields(),
            vec!["summary".to_owned(), "attachment".to_owned()]
        );
    }

    #[test]
    fn converts_transition() {
        let transition = JiraTransition::from(generated_models::Transition {
            id: Some("31".to_owned()),
            name: Some("Done".to_owned()),
            to: Some(Box::new(generated_models::Status {
                id: Some("10001".to_owned()),
                name: Some("Done".to_owned()),
            })),
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
    fn parses_jira_attachment_metadata() {
        let attachment: JiraAttachment = serde_json::from_value(serde_json::json!({
            "id": "10042",
            "filename": "error-screenshot.png",
            "mimeType": "image/png",
            "size": 2048,
            "created": "2026-05-18T10:00:00.000+0000",
            "content": "https://example.atlassian.net/rest/api/3/attachment/content/10042",
            "author": {
                "accountId": "abc",
                "displayName": "Neo",
                "active": true
            }
        }))
        .expect("attachment metadata");

        assert_eq!(attachment.id.as_deref(), Some("10042"));
        assert_eq!(attachment.filename.as_deref(), Some("error-screenshot.png"));
        assert_eq!(attachment.mime_type.as_deref(), Some("image/png"));
        assert_eq!(attachment.size, Some(2048));
        assert_eq!(
            attachment
                .author
                .as_ref()
                .and_then(|author| author.display_name.as_deref()),
            Some("Neo")
        );
    }

    #[test]
    fn builds_attachment_output_paths() {
        assert_eq!(
            attachment_output_path(None, "error.png"),
            std::path::PathBuf::from("error.png")
        );
        assert_eq!(
            attachment_output_path(Some(std::path::Path::new("downloaded.png")), "error.png"),
            std::path::PathBuf::from("downloaded.png")
        );
    }

    #[test]
    fn attachment_filename_uses_basename() {
        let attachment = JiraAttachment {
            id: Some("10042".to_owned()),
            filename: Some("../error.png".to_owned()),
            mime_type: None,
            size: None,
            author: None,
            created: None,
            content: None,
            thumbnail: None,
        };

        assert_eq!(attachment_filename(&attachment), "error.png");
    }

    #[test]
    fn converts_comment_body_to_plain_text() {
        let comment = JiraComment::from(generated_models::Comment {
            id: Some("10010".to_owned()),
            param_self: None,
            body: Some(adf_body("Line one\nLine two")),
            author: Some(Box::new(generated_models::User {
                account_id: Some("account-id".to_owned()),
                display_name: Some("Neo".to_owned()),
                active: Some(true),
            })),
            created: Some("2026-05-18T00:00:00.000+0000".to_owned()),
            updated: None,
        });

        assert_eq!(comment.id.as_deref(), Some("10010"));
        assert_eq!(comment.body_text.as_deref(), Some("Line one\nLine two"));
        assert_eq!(comment.author_display_name.as_deref(), Some("Neo"));
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
    fn resolves_assignable_user_by_exact_display_name() {
        let user = resolve_assignable_user(
            "Neo",
            vec![
                JiraUser {
                    account_id: Some("account-1".to_owned()),
                    display_name: Some("Neo".to_owned()),
                    active: Some(true),
                },
                JiraUser {
                    account_id: Some("account-2".to_owned()),
                    display_name: Some("Neon".to_owned()),
                    active: Some(true),
                },
            ],
        )
        .expect("resolved user");

        assert_eq!(user.account_id.as_deref(), Some("account-1"));
    }

    #[test]
    fn rejects_ambiguous_assignable_users() {
        let error = resolve_assignable_user(
            "neo",
            vec![
                JiraUser {
                    account_id: Some("account-1".to_owned()),
                    display_name: Some("Neo One".to_owned()),
                    active: Some(true),
                },
                JiraUser {
                    account_id: Some("account-2".to_owned()),
                    display_name: Some("Neo Two".to_owned()),
                    active: Some(true),
                },
            ],
        )
        .expect_err("ambiguous user");

        assert!(error.to_string().contains("multiple assignable Jira users"));
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
        let fields = serde_json::json!({
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
        let page = generated_models::SearchAndReconcileResults {
            is_last: Some(false),
            next_page_token: Some("next-token".to_owned()),
            issues: Some(vec![generated_models::IssueBean {
                id: Some("10002".to_owned()),
                key: Some("PROJ-1".to_owned()),
                fields: Some(fields),
            }]),
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
}
