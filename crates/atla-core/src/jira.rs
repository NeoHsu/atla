use atla_jira_api::{apis as generated_apis, models as generated_models};
use serde::{Deserialize, Serialize};

use crate::client::{ApiError, AtlassianClient};

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

    pub async fn search_issues(
        &self,
        search: &JiraIssueSearch,
    ) -> Result<JiraIssueSearchPage, ApiError> {
        let page = generated_apis::issue_search_api::search_and_reconsile_issues_using_jql(
            &self.generated,
            Some(&search.jql),
            None,
            Some(limit_i32(search.max_results)),
            Some(issue_fields()),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn get_issue(&self, issue_id_or_key: &str) -> Result<JiraIssue, ApiError> {
        generated_apis::issues_api::get_issue(
            &self.generated,
            issue_id_or_key,
            Some(issue_fields()),
        )
        .await
        .map(JiraIssue::from)
        .map_err(generated_error)
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

fn issue_fields() -> Vec<String> {
    ["summary", "status", "assignee", "issuetype", "priority"]
        .into_iter()
        .map(str::to_owned)
        .collect()
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
