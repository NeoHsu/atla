use reqwest::multipart;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::client::{ApiError, AtlassianClient, read_empty, read_json};

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

    pub async fn create_page(
        &self,
        page: &ConfluencePageCreate,
    ) -> Result<ConfluencePage, ApiError> {
        let mut request = self.client.post("/wiki/api/v2/pages");

        if let Some(private) = page.private {
            request = request.query(&[("private", private.to_string())]);
        }

        if let Some(root_level) = page.root_level {
            request = request.query(&[("root-level", root_level.to_string())]);
        }

        read_json(request.json(&page.to_payload())).await
    }

    pub async fn update_page_title(
        &self,
        id: &str,
        title: &str,
        status: ConfluenceContentStatus,
    ) -> Result<ConfluencePage, ApiError> {
        read_json(
            self.client
                .put(&format!("/wiki/api/v2/pages/{id}/title"))
                .json(&json!({
                    "status": status.as_str(),
                    "title": title,
                })),
        )
        .await
    }

    pub async fn update_page(
        &self,
        page: &ConfluencePageUpdate,
    ) -> Result<ConfluencePage, ApiError> {
        read_json(
            self.client
                .put(&format!("/wiki/api/v2/pages/{}", page.id))
                .json(&page.to_payload()),
        )
        .await
    }

    pub async fn delete_page(&self, id: &str, purge: bool, draft: bool) -> Result<(), ApiError> {
        let mut request = self.client.delete(&format!("/wiki/api/v2/pages/{id}"));

        if purge {
            request = request.query(&[("purge", "true")]);
        }

        if draft {
            request = request.query(&[("draft", "true")]);
        }

        read_empty(request).await
    }

    pub async fn move_page(&self, id: &str, parent_id: &str) -> Result<(), ApiError> {
        read_empty(
            self.client
                .put(&format!(
                    "/wiki/rest/api/content/{id}/move/append/{parent_id}"
                ))
                .json(&json!({})),
        )
        .await
    }

    pub async fn list_blog_posts(
        &self,
        search: &ConfluenceBlogPostSearch,
    ) -> Result<ConfluenceBlogPostPage, ApiError> {
        let mut request = self
            .client
            .get("/wiki/api/v2/blogposts")
            .query(&[("limit", search.limit.to_string())]);

        if let Some(space_id) = &search.space_id {
            request = request.query(&[("space-id", space_id)]);
        }

        if let Some(title) = &search.title {
            request = request.query(&[("title", title)]);
        }

        read_json(request).await
    }

    pub async fn get_blog_post(&self, id: &str) -> Result<ConfluenceBlogPost, ApiError> {
        read_json(self.client.get(&format!("/wiki/api/v2/blogposts/{id}"))).await
    }

    pub async fn create_blog_post(
        &self,
        post: &ConfluenceBlogPostCreate,
    ) -> Result<ConfluenceBlogPost, ApiError> {
        let mut request = self.client.post("/wiki/api/v2/blogposts");

        if let Some(private) = post.private {
            request = request.query(&[("private", private.to_string())]);
        }

        read_json(request.json(&post.to_payload())).await
    }

    pub async fn search(
        &self,
        search: &ConfluenceSearch,
    ) -> Result<ConfluenceSearchPage, ApiError> {
        read_json(
            self.client
                .get("/wiki/rest/api/search")
                .query(&[("cql", search.cql.as_str())])
                .query(&[("limit", search.limit.to_string())]),
        )
        .await
    }

    pub async fn list_page_attachments(
        &self,
        search: &ConfluenceAttachmentSearch,
    ) -> Result<ConfluenceAttachmentPage, ApiError> {
        let mut request = self
            .client
            .get(&format!(
                "/wiki/api/v2/pages/{}/attachments",
                search.page_id
            ))
            .query(&[("limit", search.limit.to_string())]);

        if let Some(filename) = &search.filename {
            request = request.query(&[("filename", filename)]);
        }

        read_json(request).await
    }

    pub async fn upload_page_attachment(
        &self,
        upload: ConfluenceAttachmentUpload,
    ) -> Result<ConfluenceAttachmentUploadPage, ApiError> {
        let mut part = multipart::Part::bytes(upload.bytes).file_name(upload.file_name);
        if let Some(media_type) = upload.media_type {
            part = part.mime_str(&media_type).map_err(ApiError::Request)?;
        }

        let mut form = multipart::Form::new().part("file", part);
        if let Some(comment) = upload.comment {
            form = form.text("comment", comment);
        }

        read_json(
            self.client
                .post(&format!(
                    "/wiki/rest/api/content/{}/child/attachment",
                    upload.page_id
                ))
                .header("X-Atlassian-Token", "no-check")
                .multipart(form),
        )
        .await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceBodyRepresentation {
    Storage,
    Wiki,
    AtlasDocFormat,
}

impl ConfluenceBodyRepresentation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Storage => "storage",
            Self::Wiki => "wiki",
            Self::AtlasDocFormat => "atlas_doc_format",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceContentStatus {
    Current,
    Draft,
}

impl ConfluenceContentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Draft => "draft",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluencePageCreate {
    pub space_id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub body: Option<String>,
    pub representation: ConfluenceBodyRepresentation,
    pub status: ConfluenceContentStatus,
    pub private: Option<bool>,
    pub root_level: Option<bool>,
}

impl ConfluencePageCreate {
    fn to_payload(&self) -> Value {
        let mut payload = json!({
            "spaceId": self.space_id,
            "status": self.status.as_str(),
            "title": self.title,
        });

        if let Some(parent_id) = &self.parent_id {
            payload["parentId"] = json!(parent_id);
        }

        if let Some(body) = &self.body {
            payload["body"] = json!({
                "representation": self.representation.as_str(),
                "value": body,
            });
        }

        payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluencePageUpdate {
    pub id: String,
    pub status: ConfluenceContentStatus,
    pub title: String,
    pub space_id: Option<String>,
    pub parent_id: Option<String>,
    pub body: String,
    pub representation: ConfluenceBodyRepresentation,
    pub version: u64,
    pub message: Option<String>,
}

impl ConfluencePageUpdate {
    fn to_payload(&self) -> Value {
        let mut payload = json!({
            "id": self.id,
            "status": self.status.as_str(),
            "title": self.title,
            "body": {
                "representation": self.representation.as_str(),
                "value": self.body,
            },
            "version": {
                "number": self.version,
            },
        });

        if let Some(space_id) = &self.space_id {
            payload["spaceId"] = json!(space_id);
        }

        if let Some(parent_id) = &self.parent_id {
            payload["parentId"] = json!(parent_id);
        }

        if let Some(message) = &self.message {
            payload["version"]["message"] = json!(message);
        }

        payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceBlogPostCreate {
    pub space_id: String,
    pub title: String,
    pub body: Option<String>,
    pub representation: ConfluenceBodyRepresentation,
    pub status: ConfluenceContentStatus,
    pub private: Option<bool>,
}

impl ConfluenceBlogPostCreate {
    fn to_payload(&self) -> Value {
        let mut payload = json!({
            "spaceId": self.space_id,
            "status": self.status.as_str(),
            "title": self.title,
        });

        if let Some(body) = &self.body {
            payload["body"] = json!({
                "representation": self.representation.as_str(),
                "value": body,
            });
        }

        payload
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceBlogPostSearch {
    pub space_id: Option<String>,
    pub title: Option<String>,
    pub limit: u32,
}

impl Default for ConfluenceBlogPostSearch {
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
pub struct ConfluenceBlogPostPage {
    #[serde(default)]
    pub results: Vec<ConfluenceBlogPost>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceBlogPost {
    pub id: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub space_id: Option<String>,
    pub author_id: Option<String>,
    pub created_at: Option<String>,
    pub version: Option<ConfluenceVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceSearch {
    pub cql: String,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSearchPage {
    #[serde(default)]
    pub results: Vec<ConfluenceSearchResult>,
    pub size: Option<u64>,
    pub total_size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSearchResult {
    pub title: Option<String>,
    pub url: Option<String>,
    pub excerpt: Option<String>,
    pub content: Option<ConfluenceSearchContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSearchContent {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceAttachmentSearch {
    pub page_id: String,
    pub filename: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachmentPage {
    #[serde(default)]
    pub results: Vec<ConfluenceAttachment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachment {
    pub id: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub page_id: Option<String>,
    pub blog_post_id: Option<String>,
    pub media_type: Option<String>,
    pub media_type_description: Option<String>,
    pub file_id: Option<String>,
    pub file_size: Option<i64>,
    pub webui_link: Option<String>,
    pub download_link: Option<String>,
    pub version: Option<ConfluenceVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceAttachmentUpload {
    pub page_id: String,
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub media_type: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachmentUploadPage {
    #[serde(default)]
    pub results: Vec<ConfluenceAttachmentUploadResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachmentUploadResult {
    pub id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub version: Option<ConfluenceVersion>,
    pub metadata: Option<ConfluenceAttachmentMetadata>,
    pub extensions: Option<ConfluenceAttachmentExtensions>,
    #[serde(rename = "_links")]
    pub links: Option<ConfluenceAttachmentUploadLinks>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachmentMetadata {
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachmentExtensions {
    pub file_size: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachmentUploadLinks {
    pub download: Option<String>,
    pub webui: Option<String>,
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

    #[test]
    fn parses_blog_post_page() {
        let page: ConfluenceBlogPostPage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "222",
                        "status": "current",
                        "title": "Release Notes",
                        "spaceId": "12345",
                        "authorId": "abc",
                        "createdAt": "2026-05-17T00:00:00.000Z",
                        "version": {
                            "number": 2,
                            "message": "Publish",
                            "createdAt": "2026-05-17T01:00:00.000Z"
                        }
                    }
                ]
            }"#,
        )
        .expect("parse blog post page");

        let post = &page.results[0];
        assert_eq!(post.id.as_deref(), Some("222"));
        assert_eq!(post.title.as_deref(), Some("Release Notes"));
        assert_eq!(post.space_id.as_deref(), Some("12345"));
        assert_eq!(
            post.version.as_ref().and_then(|version| version.number),
            Some(2)
        );
    }

    #[test]
    fn builds_create_page_payload() {
        let payload = ConfluencePageCreate {
            space_id: "12345".to_owned(),
            title: "Meeting Notes".to_owned(),
            parent_id: Some("100".to_owned()),
            body: Some("<p>Hello</p>".to_owned()),
            representation: ConfluenceBodyRepresentation::Storage,
            status: ConfluenceContentStatus::Current,
            private: None,
            root_level: None,
        }
        .to_payload();

        assert_eq!(payload["spaceId"], "12345");
        assert_eq!(payload["title"], "Meeting Notes");
        assert_eq!(payload["parentId"], "100");
        assert_eq!(payload["body"]["representation"], "storage");
        assert_eq!(payload["body"]["value"], "<p>Hello</p>");
    }

    #[test]
    fn parses_search_page() {
        let page: ConfluenceSearchPage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "title": "Runbook",
                        "url": "/wiki/spaces/DEV/pages/111/Runbook",
                        "content": {
                            "id": "111",
                            "type": "page",
                            "status": "current",
                            "title": "Runbook"
                        }
                    }
                ],
                "size": 1,
                "totalSize": 1
            }"#,
        )
        .expect("parse search page");

        let result = &page.results[0];
        assert_eq!(result.title.as_deref(), Some("Runbook"));
        assert_eq!(
            result
                .content
                .as_ref()
                .and_then(|content| content.content_type.as_deref()),
            Some("page")
        );
    }

    #[test]
    fn parses_attachment_page() {
        let page: ConfluenceAttachmentPage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "att123",
                        "status": "current",
                        "title": "diagram.png",
                        "pageId": "111",
                        "mediaType": "image/png",
                        "fileSize": 2048,
                        "downloadLink": "/download/attachments/111/diagram.png",
                        "version": {
                            "number": 1
                        }
                    }
                ]
            }"#,
        )
        .expect("parse attachment page");

        let attachment = &page.results[0];
        assert_eq!(attachment.id.as_deref(), Some("att123"));
        assert_eq!(attachment.media_type.as_deref(), Some("image/png"));
        assert_eq!(attachment.file_size, Some(2048));
    }
}
