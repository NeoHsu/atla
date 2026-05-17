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
}
