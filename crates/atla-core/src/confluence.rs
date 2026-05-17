use serde::{Deserialize, Serialize};

use crate::client::{ApiError, AtlassianClient, read_json};

#[derive(Debug, Clone)]
pub struct ConfluenceClient {
    client: AtlassianClient,
}

impl ConfluenceClient {
    pub fn new(client: AtlassianClient) -> Self {
        Self { client }
    }

    pub fn instance_url(&self) -> &str {
        &self.client.instance().base_url
    }

    pub async fn list_spaces(
        &self,
        search: &ConfluenceSpaceSearch,
    ) -> Result<ConfluenceSpacePage, ApiError> {
        let mut request = self
            .client
            .get("/wiki/api/v2/spaces")
            .query(&[("limit", search.limit.to_string())]);

        if let Some(key) = &search.key {
            request = request.query(&[("keys", key)]);
        }

        read_json(request).await
    }

    pub async fn get_space_by_key(&self, key: &str) -> Result<Option<ConfluenceSpace>, ApiError> {
        let page = self
            .list_spaces(&ConfluenceSpaceSearch {
                key: Some(key.to_owned()),
                limit: 1,
            })
            .await?;

        Ok(page.results.into_iter().next())
    }

    pub async fn list_pages(
        &self,
        search: &ConfluencePageSearch,
    ) -> Result<ConfluencePagePage, ApiError> {
        let mut request = self
            .client
            .get("/wiki/api/v2/pages")
            .query(&[("limit", search.limit.to_string())]);

        if let Some(space_id) = &search.space_id {
            request = request.query(&[("space-id", space_id)]);
        }

        if let Some(title) = &search.title {
            request = request.query(&[("title", title)]);
        }

        read_json(request).await
    }

    pub async fn get_page(&self, id: &str) -> Result<ConfluencePage, ApiError> {
        read_json(self.client.get(&format!("/wiki/api/v2/pages/{id}"))).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceSpaceSearch {
    pub key: Option<String>,
    pub limit: u32,
}

impl Default for ConfluenceSpaceSearch {
    fn default() -> Self {
        Self {
            key: None,
            limit: 25,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSpacePage {
    #[serde(default)]
    pub results: Vec<ConfluenceSpace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSpace {
    pub id: Option<String>,
    pub key: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub space_type: Option<String>,
    pub status: Option<String>,
    pub homepage_id: Option<String>,
    pub current_active_alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluencePageSearch {
    pub space_id: Option<String>,
    pub title: Option<String>,
    pub limit: u32,
}

impl Default for ConfluencePageSearch {
    fn default() -> Self {
        Self {
            space_id: None,
            title: None,
            limit: 25,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluencePagePage {
    #[serde(default)]
    pub results: Vec<ConfluencePage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluencePage {
    pub id: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub space_id: Option<String>,
    pub parent_id: Option<String>,
    pub author_id: Option<String>,
    pub owner_id: Option<String>,
    pub created_at: Option<String>,
    pub version: Option<ConfluenceVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceVersion {
    pub number: Option<u64>,
    pub message: Option<String>,
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_space_page() {
        let page: ConfluenceSpacePage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "12345",
                        "key": "DEV",
                        "name": "Development",
                        "type": "global",
                        "status": "current",
                        "homepageId": "67890",
                        "currentActiveAlias": "DEV"
                    }
                ]
            }"#,
        )
        .expect("parse space page");

        assert_eq!(page.results[0].key.as_deref(), Some("DEV"));
        assert_eq!(page.results[0].space_type.as_deref(), Some("global"));
        assert_eq!(page.results[0].homepage_id.as_deref(), Some("67890"));
    }

    #[test]
    fn parses_page_page() {
        let page: ConfluencePagePage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "111",
                        "status": "current",
                        "title": "Runbook",
                        "spaceId": "12345",
                        "parentId": "100",
                        "authorId": "abc",
                        "createdAt": "2026-05-17T00:00:00.000Z",
                        "version": {
                            "number": 3,
                            "message": "Update",
                            "createdAt": "2026-05-17T01:00:00.000Z"
                        }
                    }
                ]
            }"#,
        )
        .expect("parse page page");

        let page = &page.results[0];
        assert_eq!(page.id.as_deref(), Some("111"));
        assert_eq!(page.title.as_deref(), Some("Runbook"));
        assert_eq!(page.space_id.as_deref(), Some("12345"));
        assert_eq!(
            page.version.as_ref().and_then(|version| version.number),
            Some(3)
        );
    }
}
