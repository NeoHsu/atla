use serde::{Deserialize, Serialize};

use crate::client::{ApiError, AtlassianClient, read_json};

#[derive(Debug, Clone)]
pub struct JiraClient {
    client: AtlassianClient,
}

impl JiraClient {
    pub fn new(client: AtlassianClient) -> Self {
        Self { client }
    }

    pub fn instance_url(&self) -> &str {
        &self.client.instance().base_url
    }

    pub async fn search_projects(
        &self,
        search: &JiraProjectSearch,
    ) -> Result<JiraProjectPage, ApiError> {
        let mut request = self.client.get("/rest/api/3/project/search").query(&[
            ("startAt", search.start_at.to_string()),
            ("maxResults", search.max_results.to_string()),
        ]);

        if let Some(query) = &search.query {
            request = request.query(&[("query", query)]);
        }

        read_json(request).await
    }

    pub async fn get_project(&self, project_id_or_key: &str) -> Result<JiraProject, ApiError> {
        read_json(
            self.client
                .get(&format!("/rest/api/3/project/{project_id_or_key}")),
        )
        .await
    }

    pub async fn search_issues(
        &self,
        search: &JiraIssueSearch,
    ) -> Result<JiraIssueSearchPage, ApiError> {
        let query = [
            ("jql", search.jql.clone()),
            ("maxResults", search.max_results.to_string()),
            (
                "fields",
                "summary,status,assignee,issuetype,priority".to_owned(),
            ),
        ];
        let request = self.client.get("/rest/api/3/search/jql").query(&query);

        read_json(request).await
    }

    pub async fn get_issue(&self, issue_id_or_key: &str) -> Result<JiraIssue, ApiError> {
        let query = [("fields", "summary,status,assignee,issuetype,priority")];
        let request = self
            .client
            .get(&format!("/rest/api/3/issue/{issue_id_or_key}"))
            .query(&query);

        read_json(request).await
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
pub struct JiraIssueSearchPage {
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub issues: Vec<JiraIssue>,
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
