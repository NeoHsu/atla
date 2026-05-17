use atla_confluence_api::{apis as generated_apis, models as generated_models};
use serde::{Deserialize, Serialize};

use crate::client::{ApiError, AtlassianClient, read_json};

#[derive(Debug, Clone)]
pub struct ConfluenceClient {
    raw_client: AtlassianClient,
    generated: generated_apis::configuration::Configuration,
}

impl ConfluenceClient {
    pub fn new(client: AtlassianClient) -> Self {
        let generated = generated_apis::configuration::Configuration {
            base_path: format!("{}/wiki/api/v2", client.instance().base_url),
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

    pub async fn list_spaces(
        &self,
        search: &ConfluenceSpaceSearch,
    ) -> Result<ConfluenceSpacePage, ApiError> {
        let keys = search.key.as_ref().map(|key| vec![key.clone()]);
        let page = generated_apis::space_api::get_spaces(
            &self.generated,
            None,
            keys,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluenceSpacePage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceSpace::from)
                .collect(),
        })
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
        let space_id = optional_i64_vec(search.space_id.as_deref())?;
        let page = generated_apis::page_api::get_pages(
            &self.generated,
            None,
            space_id,
            None,
            None,
            search.title.as_deref(),
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluencePagePage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluencePage::from)
                .collect(),
        })
    }

    pub async fn get_page(&self, id: &str) -> Result<ConfluencePage, ApiError> {
        let page = generated_apis::page_api::get_page_by_id(
            &self.generated,
            parse_i64_id(id)?,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn create_page(
        &self,
        page: &ConfluencePageCreate,
    ) -> Result<ConfluencePage, ApiError> {
        let page = generated_apis::page_api::create_page(
            &self.generated,
            page.to_generated(),
            None,
            page.private,
            page.root_level,
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn update_page_title(
        &self,
        id: &str,
        title: &str,
        status: ConfluenceContentStatus,
    ) -> Result<ConfluencePage, ApiError> {
        let page = generated_apis::page_api::update_page_title(
            &self.generated,
            parse_i64_id(id)?,
            generated_models::UpdatePageTitleRequest::new(
                status.into_update_page_title_status(),
                title.to_owned(),
            ),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn update_page(
        &self,
        page: &ConfluencePageUpdate,
    ) -> Result<ConfluencePage, ApiError> {
        let updated = generated_apis::page_api::update_page(
            &self.generated,
            parse_i64_id(&page.id)?,
            page.to_generated(),
        )
        .await
        .map_err(generated_error)?;

        Ok(updated.into())
    }

    pub async fn delete_page(&self, id: &str, purge: bool, draft: bool) -> Result<(), ApiError> {
        generated_apis::page_api::delete_page(
            &self.generated,
            parse_i64_id(id)?,
            Some(purge),
            Some(draft),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn move_page(&self, id: &str, parent_id: &str) -> Result<ConfluencePage, ApiError> {
        let existing = generated_apis::page_api::get_page_by_id(
            &self.generated,
            parse_i64_id(id)?,
            Some(generated_models::PrimaryBodyRepresentationSingle::Storage),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(generated_error)?;

        let body = existing
            .body
            .as_ref()
            .and_then(|body| body.storage.as_ref())
            .and_then(|body| body.value.clone())
            .ok_or_else(|| {
                ApiError::Decode(format!("page `{id}` did not include a storage body"))
            })?;
        let title = existing
            .title
            .clone()
            .ok_or_else(|| ApiError::Decode(format!("page `{id}` did not include a title")))?;
        let version = existing
            .version
            .as_ref()
            .and_then(|version| version.number)
            .ok_or_else(|| ApiError::Decode(format!("page `{id}` did not include a version")))?;
        let status = match existing.status {
            Some(generated_models::ContentStatus::Draft) => ConfluenceContentStatus::Draft,
            _ => ConfluenceContentStatus::Current,
        };

        self.update_page(&ConfluencePageUpdate {
            id: id.to_owned(),
            status,
            title,
            space_id: existing.space_id,
            parent_id: Some(parent_id.to_owned()),
            body,
            representation: ConfluenceBodyRepresentation::Storage,
            version: version as u64 + 1,
            message: Some(format!("Move under {parent_id}")),
        })
        .await
    }

    pub async fn list_blog_posts(
        &self,
        search: &ConfluenceBlogPostSearch,
    ) -> Result<ConfluenceBlogPostPage, ApiError> {
        let space_id = optional_i64_vec(search.space_id.as_deref())?;
        let page = generated_apis::blog_post_api::get_blog_posts(
            &self.generated,
            None,
            space_id,
            None,
            None,
            search.title.as_deref(),
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluenceBlogPostPage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceBlogPost::from)
                .collect(),
        })
    }

    pub async fn get_blog_post(&self, id: &str) -> Result<ConfluenceBlogPost, ApiError> {
        let post = generated_apis::blog_post_api::get_blog_post_by_id(
            &self.generated,
            parse_i64_id(id)?,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        )
        .await
        .map_err(generated_error)?;

        Ok(post.into())
    }

    pub async fn create_blog_post(
        &self,
        post: &ConfluenceBlogPostCreate,
    ) -> Result<ConfluenceBlogPost, ApiError> {
        let post = generated_apis::blog_post_api::create_blog_post(
            &self.generated,
            post.to_generated(),
            post.private,
        )
        .await
        .map_err(generated_error)?;

        Ok(post.into())
    }

    pub async fn search(
        &self,
        search: &ConfluenceSearch,
    ) -> Result<ConfluenceSearchPage, ApiError> {
        read_json(
            self.raw_client
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
        let page = generated_apis::attachment_api::get_page_attachments(
            &self.generated,
            parse_i64_id(&search.page_id)?,
            None,
            None,
            None,
            None,
            search.filename.as_deref(),
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluenceAttachmentPage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceAttachment::from)
                .collect(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceBodyRepresentation {
    Storage,
    Wiki,
    AtlasDocFormat,
}

impl ConfluenceBodyRepresentation {
    fn as_page_body_write(self) -> generated_models::page_body_write::Representation {
        match self {
            Self::Storage => generated_models::page_body_write::Representation::Storage,
            Self::Wiki => generated_models::page_body_write::Representation::Wiki,
            Self::AtlasDocFormat => {
                generated_models::page_body_write::Representation::AtlasDocFormat
            }
        }
    }

    fn as_blog_post_body_write(self) -> generated_models::blog_post_body_write::Representation {
        match self {
            Self::Storage => generated_models::blog_post_body_write::Representation::Storage,
            Self::Wiki => generated_models::blog_post_body_write::Representation::Wiki,
            Self::AtlasDocFormat => {
                generated_models::blog_post_body_write::Representation::AtlasDocFormat
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceContentStatus {
    Current,
    Draft,
}

impl ConfluenceContentStatus {
    fn into_create_page_status(self) -> generated_models::create_page_request::Status {
        match self {
            Self::Current => generated_models::create_page_request::Status::Current,
            Self::Draft => generated_models::create_page_request::Status::Draft,
        }
    }

    fn into_update_page_status(self) -> generated_models::update_page_request::Status {
        match self {
            Self::Current => generated_models::update_page_request::Status::Current,
            Self::Draft => generated_models::update_page_request::Status::Draft,
        }
    }

    fn into_update_page_title_status(self) -> generated_models::update_page_title_request::Status {
        match self {
            Self::Current => generated_models::update_page_title_request::Status::Current,
            Self::Draft => generated_models::update_page_title_request::Status::Draft,
        }
    }

    fn into_create_blog_post_status(self) -> generated_models::create_blog_post_request::Status {
        match self {
            Self::Current => generated_models::create_blog_post_request::Status::Current,
            Self::Draft => generated_models::create_blog_post_request::Status::Draft,
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
    fn to_generated(&self) -> generated_models::CreatePageRequest {
        let mut request = generated_models::CreatePageRequest::new(self.space_id.clone());
        request.status = Some(self.status.into_create_page_status());
        request.title = Some(self.title.clone());
        request.parent_id.clone_from(&self.parent_id);
        request.body = self.body.as_ref().map(|body| {
            Box::new(generated_models::CreatePageRequestBody::PageBodyWrite(
                Box::new(generated_models::PageBodyWrite {
                    representation: Some(self.representation.as_page_body_write()),
                    value: Some(body.clone()),
                }),
            ))
        });
        request
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
    fn to_generated(&self) -> generated_models::UpdatePageRequest {
        let mut version = generated_models::UpdatePageRequestVersion::new();
        version.number = Some(self.version as i32);
        version.message.clone_from(&self.message);

        let mut request = generated_models::UpdatePageRequest::new(
            self.id.clone(),
            self.status.into_update_page_status(),
            self.title.clone(),
            generated_models::CreatePageRequestBody::PageBodyWrite(Box::new(
                generated_models::PageBodyWrite {
                    representation: Some(self.representation.as_page_body_write()),
                    value: Some(self.body.clone()),
                },
            )),
            version,
        );

        request.space_id = self
            .space_id
            .as_ref()
            .map(|space_id| Some(serde_json::Value::String(space_id.clone())));
        request.parent_id = self
            .parent_id
            .as_ref()
            .map(|parent_id| Some(serde_json::Value::String(parent_id.clone())));
        request
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
    fn to_generated(&self) -> generated_models::CreateBlogPostRequest {
        let mut request = generated_models::CreateBlogPostRequest::new(self.space_id.clone());
        request.status = Some(self.status.into_create_blog_post_status());
        request.title = Some(self.title.clone());
        request.body = self.body.as_ref().map(|body| {
            Box::new(
                generated_models::CreateBlogPostRequestBody::BlogPostBodyWrite(Box::new(
                    generated_models::BlogPostBodyWrite {
                        representation: Some(self.representation.as_blog_post_body_write()),
                        value: Some(body.clone()),
                    },
                )),
            )
        });
        request
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
    pub body: Option<String>,
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

fn limit_i32(limit: u32) -> i32 {
    limit.min(i32::MAX as u32) as i32
}

fn parse_i64_id(id: &str) -> Result<i64, ApiError> {
    id.parse()
        .map_err(|_| ApiError::Decode(format!("expected numeric Confluence id, got `{id}`")))
}

fn optional_i64_vec(id: Option<&str>) -> Result<Option<Vec<i64>>, ApiError> {
    id.map(|id| parse_i64_id(id).map(|id| vec![id])).transpose()
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

fn version_from_generated(
    version: Option<Box<generated_models::Version>>,
) -> Option<ConfluenceVersion> {
    version.map(|version| ConfluenceVersion {
        number: version.number.map(|number| number as u64),
        message: version.message,
        created_at: version.created_at.map(|created_at| created_at.to_rfc3339()),
    })
}

fn body_from_generated(body: Option<Box<generated_models::BodySingle>>) -> Option<String> {
    body.and_then(|body| {
        let body = *body;
        body.storage
            .and_then(|body| body.value)
            .or_else(|| body.atlas_doc_format.and_then(|body| body.value))
            .or_else(|| body.view.and_then(|body| body.value))
    })
}

impl From<generated_models::SpaceBulk> for ConfluenceSpace {
    fn from(space: generated_models::SpaceBulk) -> Self {
        Self {
            id: space.id,
            key: space.key,
            name: space.name,
            space_type: space.r#type.map(|space_type| space_type.to_string()),
            status: space.status.map(|status| status.to_string()),
            homepage_id: space.homepage_id,
            current_active_alias: space.current_active_alias,
        }
    }
}

impl From<generated_models::PageBulk> for ConfluencePage {
    fn from(page: generated_models::PageBulk) -> Self {
        Self {
            id: page.id,
            status: page.status.map(|status| status.to_string()),
            title: page.title,
            space_id: page.space_id,
            parent_id: page.parent_id,
            author_id: page.author_id,
            owner_id: page.owner_id.flatten(),
            created_at: page.created_at.map(|created_at| created_at.to_rfc3339()),
            version: version_from_generated(page.version),
            body: None,
        }
    }
}

impl From<generated_models::CreatePage200Response> for ConfluencePage {
    fn from(page: generated_models::CreatePage200Response) -> Self {
        Self {
            id: page.id,
            status: page.status.map(|status| status.to_string()),
            title: page.title,
            space_id: page.space_id,
            parent_id: page.parent_id,
            author_id: page.author_id,
            owner_id: page.owner_id.flatten(),
            created_at: page.created_at.map(|created_at| created_at.to_rfc3339()),
            version: version_from_generated(page.version),
            body: body_from_generated(page.body),
        }
    }
}

impl From<generated_models::BlogPostBulk> for ConfluenceBlogPost {
    fn from(post: generated_models::BlogPostBulk) -> Self {
        Self {
            id: post.id,
            status: post.status.map(|status| status.to_string()),
            title: post.title,
            space_id: post.space_id,
            author_id: post.author_id,
            created_at: post.created_at.map(|created_at| created_at.to_rfc3339()),
            version: version_from_generated(post.version),
        }
    }
}

impl From<generated_models::CreateBlogPost200Response> for ConfluenceBlogPost {
    fn from(post: generated_models::CreateBlogPost200Response) -> Self {
        Self {
            id: post.id,
            status: post.status.map(|status| status.to_string()),
            title: post.title,
            space_id: post.space_id,
            author_id: post.author_id,
            created_at: post.created_at.map(|created_at| created_at.to_rfc3339()),
            version: version_from_generated(post.version),
        }
    }
}

impl From<generated_models::AttachmentBulk> for ConfluenceAttachment {
    fn from(attachment: generated_models::AttachmentBulk) -> Self {
        Self {
            id: attachment.id,
            status: attachment.status.map(|status| status.to_string()),
            title: attachment.title,
            page_id: attachment.page_id,
            blog_post_id: attachment.blog_post_id,
            media_type: attachment.media_type,
            media_type_description: attachment.media_type_description,
            file_id: attachment.file_id,
            file_size: attachment.file_size,
            webui_link: attachment.webui_link,
            download_link: attachment.download_link,
            version: version_from_generated(attachment.version),
        }
    }
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
    fn builds_generated_create_page_request() {
        let request = ConfluencePageCreate {
            space_id: "12345".to_owned(),
            title: "Meeting Notes".to_owned(),
            parent_id: Some("100".to_owned()),
            body: Some("<p>Hello</p>".to_owned()),
            representation: ConfluenceBodyRepresentation::Storage,
            status: ConfluenceContentStatus::Current,
            private: None,
            root_level: None,
        }
        .to_generated();

        assert_eq!(request.space_id, "12345");
        assert_eq!(request.title.as_deref(), Some("Meeting Notes"));
        assert_eq!(request.parent_id.as_deref(), Some("100"));

        let Some(body) = request.body else {
            panic!("expected generated body");
        };
        let generated_models::CreatePageRequestBody::PageBodyWrite(body) = *body else {
            panic!("expected page body write");
        };
        assert_eq!(body.value.as_deref(), Some("<p>Hello</p>"));
        assert_eq!(
            body.representation,
            Some(generated_models::page_body_write::Representation::Storage)
        );
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
