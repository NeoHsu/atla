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
}
