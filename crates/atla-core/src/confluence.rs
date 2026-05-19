use atla_confluence_api::{apis as generated_apis, models as generated_models};
use atla_confluence_v1_api::{apis as generated_v1_apis, models as generated_v1_models};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::client::{ApiError, AtlassianClient};

#[derive(Debug, Clone)]
pub struct ConfluenceClient {
    raw_client: AtlassianClient,
    generated: generated_apis::configuration::Configuration,
    generated_v1: generated_v1_apis::configuration::Configuration,
}

impl ConfluenceClient {
    pub fn new(client: AtlassianClient) -> Self {
        let generated = generated_apis::configuration::Configuration {
            base_path: format!("{}/wiki/api/v2", client.instance().base_url),
            user_agent: Some("atla".to_owned()),
            basic_auth: Some((client.email().to_owned(), Some(client.token().to_owned()))),
            ..Default::default()
        };
        let generated_v1 = generated_v1_apis::configuration::Configuration {
            base_path: client.instance().base_url.clone(),
            user_agent: Some("atla".to_owned()),
            basic_auth: Some((client.email().to_owned(), Some(client.token().to_owned()))),
            ..Default::default()
        };

        Self {
            raw_client: client,
            generated,
            generated_v1,
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

    pub async fn create_space(
        &self,
        space: &ConfluenceSpaceCreate,
    ) -> Result<ConfluenceSpace, ApiError> {
        if space.key.is_none() && space.alias.is_none() {
            return Err(ApiError::Decode(
                "Confluence space create requires a key or alias".to_owned(),
            ));
        }

        generated_apis::space_api::create_space(&self.generated, space.to_generated())
            .await
            .map(ConfluenceSpace::from)
            .map_err(generated_error)
    }

    pub async fn update_space(
        &self,
        space: &ConfluenceSpaceUpdate,
    ) -> Result<ConfluenceSpace, ApiError> {
        if space.name.is_none() && space.description.is_none() {
            return Err(ApiError::Decode(
                "Confluence space update requires at least one field".to_owned(),
            ));
        }

        let _space = generated_v1_apis::space_api::update_space(
            &self.generated_v1,
            &space.key,
            space.to_v1_update_request(),
        )
        .await
        .map_err(generated_v1_error)?;

        self.get_space_by_key(&space.key).await?.ok_or_else(|| {
            ApiError::Decode(format!(
                "Confluence space `{}` was updated but could not be loaded",
                space.key
            ))
        })
    }

    pub async fn delete_space(&self, key: &str) -> Result<(), ApiError> {
        let _task = generated_v1_apis::space_api::delete_space(&self.generated_v1, key)
            .await
            .map_err(generated_v1_error)?;
        Ok(())
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

    pub async fn list_page_children(
        &self,
        search: &ConfluenceContentTreeSearch,
    ) -> Result<ConfluenceContentTreePage, ApiError> {
        let id = parse_i64_id(&search.page_id)?;
        if let Some(depth) = search.depth {
            let page = generated_apis::descendants_api::get_page_descendants(
                &self.generated,
                id,
                Some(limit_i32(search.limit)),
                Some(limit_i32(depth)),
                None,
            )
            .await
            .map_err(generated_error)?;

            return Ok(ConfluenceContentTreePage {
                results: page
                    .results
                    .unwrap_or_default()
                    .into_iter()
                    .map(ConfluenceContentNode::from)
                    .collect(),
            });
        }

        let page = generated_apis::children_api::get_page_direct_children(
            &self.generated,
            id,
            None,
            Some(limit_i32(search.limit)),
            None,
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluenceContentTreePage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceContentNode::from)
                .collect(),
        })
    }

    pub async fn get_page(&self, id: &str) -> Result<ConfluencePage, ApiError> {
        self.get_page_with_body_format(id, None).await
    }

    pub async fn get_page_with_body_format(
        &self,
        id: &str,
        body_format: Option<ConfluenceBodyRepresentation>,
    ) -> Result<ConfluencePage, ApiError> {
        let page = generated_apis::page_api::get_page_by_id(
            &self.generated,
            parse_i64_id(id)?,
            body_format.map(ConfluenceBodyRepresentation::as_primary_body_single),
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

    pub async fn copy_page(&self, copy: &ConfluencePageCopy) -> Result<ConfluencePage, ApiError> {
        let source = self
            .get_page_with_body_format(&copy.source_id, Some(ConfluenceBodyRepresentation::Storage))
            .await?;
        let body = source.body.ok_or_else(|| {
            ApiError::Decode(format!(
                "Confluence page `{}` did not include storage body",
                copy.source_id
            ))
        })?;
        let space_id = copy.space_id.clone().or(source.space_id).ok_or_else(|| {
            ApiError::Decode(format!(
                "Confluence page `{}` did not include a space id; pass --space-id",
                copy.source_id
            ))
        })?;

        self.create_page(&ConfluencePageCreate {
            space_id,
            title: copy.title.clone(),
            parent_id: copy.parent_id.clone(),
            body: Some(body),
            representation: ConfluenceBodyRepresentation::Storage,
            status: ConfluenceContentStatus::Current,
            private: None,
            root_level: copy.root_level.then_some(true),
        })
        .await
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

    pub async fn update_blog_post(
        &self,
        post: &ConfluenceBlogPostUpdate,
    ) -> Result<ConfluenceBlogPost, ApiError> {
        let post = generated_apis::blog_post_api::update_blog_post(
            &self.generated,
            parse_i64_id(&post.id)?,
            post.to_generated(),
        )
        .await
        .map_err(generated_error)?;

        Ok(post.into())
    }

    pub async fn delete_blog_post(
        &self,
        id: &str,
        purge: bool,
        draft: bool,
    ) -> Result<(), ApiError> {
        generated_apis::blog_post_api::delete_blog_post(
            &self.generated,
            parse_i64_id(id)?,
            Some(purge),
            Some(draft),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn list_page_labels(
        &self,
        search: &ConfluenceLabelSearch,
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let page = generated_apis::label_api::get_page_labels(
            &self.generated,
            parse_i64_id(&search.content_id)?,
            search.prefix.as_deref(),
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn list_blog_labels(
        &self,
        search: &ConfluenceLabelSearch,
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let page = generated_apis::label_api::get_blog_post_labels(
            &self.generated,
            parse_i64_id(&search.content_id)?,
            search.prefix.as_deref(),
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn add_page_labels(
        &self,
        content_id: &str,
        labels: &[String],
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let request = generated_v1_models::AddLabelsToContentRequest::LabelCreateArray(
            labels
                .iter()
                .map(|label| {
                    generated_v1_models::LabelCreate::new("global".to_owned(), label.clone())
                })
                .collect(),
        );
        let _labels = generated_v1_apis::content_labels_api::add_labels_to_content(
            &self.generated_v1,
            content_id,
            request,
        )
        .await
        .map_err(generated_v1_error)?;

        Ok(ConfluenceLabelPage {
            results: labels
                .iter()
                .map(|label| ConfluenceLabel {
                    id: None,
                    name: Some(label.clone()),
                    prefix: Some("global".to_owned()),
                })
                .collect(),
        })
    }

    pub async fn remove_page_label(&self, content_id: &str, label: &str) -> Result<(), ApiError> {
        generated_v1_apis::content_labels_api::remove_label_from_content_using_query_parameter(
            &self.generated_v1,
            content_id,
            label,
        )
        .await
        .map_err(generated_v1_error)
    }

    pub async fn add_blog_labels(
        &self,
        content_id: &str,
        labels: &[String],
    ) -> Result<ConfluenceLabelPage, ApiError> {
        self.add_page_labels(content_id, labels).await
    }

    pub async fn remove_blog_label(&self, content_id: &str, label: &str) -> Result<(), ApiError> {
        self.remove_page_label(content_id, label).await
    }

    pub async fn list_page_comments(
        &self,
        search: &ConfluenceCommentSearch,
    ) -> Result<ConfluenceCommentPage, ApiError> {
        let page = generated_apis::comment_api::get_page_footer_comments(
            &self.generated,
            parse_i64_id(&search.content_id)?,
            Some(generated_models::PrimaryBodyRepresentation::Storage),
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn add_page_comment(
        &self,
        comment: &ConfluenceCommentCreate,
    ) -> Result<ConfluenceComment, ApiError> {
        let created = generated_apis::comment_api::create_footer_comment(
            &self.generated,
            comment.to_generated_page_footer(),
        )
        .await
        .map_err(generated_error)?;

        Ok(created.into())
    }

    pub async fn delete_page_comment(&self, comment_id: &str) -> Result<(), ApiError> {
        generated_apis::comment_api::delete_footer_comment(
            &self.generated,
            parse_i64_id(comment_id)?,
        )
        .await
        .map_err(generated_error)
    }

    pub async fn list_blog_comments(
        &self,
        search: &ConfluenceCommentSearch,
    ) -> Result<ConfluenceCommentPage, ApiError> {
        let page = generated_apis::comment_api::get_blog_post_footer_comments(
            &self.generated,
            parse_i64_id(&search.content_id)?,
            Some(generated_models::PrimaryBodyRepresentation::Storage),
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn add_blog_comment(
        &self,
        comment: &ConfluenceCommentCreate,
    ) -> Result<ConfluenceComment, ApiError> {
        let created = generated_apis::comment_api::create_footer_comment(
            &self.generated,
            comment.to_generated_blog_footer(),
        )
        .await
        .map_err(generated_error)?;

        Ok(created.into())
    }

    pub async fn search(
        &self,
        search: &ConfluenceSearch,
    ) -> Result<ConfluenceSearchPage, ApiError> {
        let page = generated_v1_apis::search_api::search_by_cql(
            &self.generated_v1,
            &search.cql,
            None,
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(generated_v1_error)?;

        Ok(page.into())
    }

    pub async fn upload_page_attachment(
        &self,
        upload: &ConfluenceAttachmentUpload,
    ) -> Result<ConfluenceAttachmentPage, ApiError> {
        let page = generated_v1_apis::content_attachments_api::create_or_update_attachments(
            &self.generated_v1,
            &upload.page_id,
            "nocheck",
            upload.file.clone(),
            upload.minor_edit,
            Some("current"),
            upload.comment.as_deref(),
        )
        .await
        .map_err(generated_v1_error)?;

        Ok(ConfluenceAttachmentPage {
            results: page
                .results
                .into_iter()
                .map(ConfluenceAttachment::from)
                .collect(),
        })
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

    pub async fn get_attachment(&self, id: &str) -> Result<ConfluenceAttachment, ApiError> {
        generated_apis::attachment_api::get_attachment_by_id(
            &self.generated,
            id,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
        )
        .await
        .map(ConfluenceAttachment::from)
        .map_err(generated_error)
    }

    pub async fn delete_attachment(&self, id: &str, purge: bool) -> Result<(), ApiError> {
        generated_apis::attachment_api::delete_attachment(
            &self.generated,
            parse_i64_id(id)?,
            Some(purge),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn download_attachment(
        &self,
        id: &str,
        output: Option<&Path>,
    ) -> Result<ConfluenceAttachmentDownload, ApiError> {
        let attachment = self.get_attachment(id).await?;
        let download_link = attachment.download_link.clone().ok_or_else(|| {
            ApiError::Decode(format!("attachment `{id}` did not include a downloadLink"))
        })?;
        let response = self.raw_client.get(&download_link).send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Http {
                status,
                body: crate::client::extract_api_error_body(&body),
            });
        }
        let bytes = response.bytes().await.map_err(ApiError::Request)?;
        let filename = output
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(attachment.title.as_deref().unwrap_or(id)));
        std::fs::write(&filename, &bytes).map_err(|error| {
            ApiError::Decode(format!("failed to write {}: {error}", filename.display()))
        })?;

        Ok(ConfluenceAttachmentDownload {
            attachment,
            path: filename,
            bytes: bytes.len() as u64,
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
    fn as_primary_body_single(self) -> generated_models::PrimaryBodyRepresentationSingle {
        match self {
            Self::Storage | Self::Wiki => {
                generated_models::PrimaryBodyRepresentationSingle::Storage
            }
            Self::AtlasDocFormat => {
                generated_models::PrimaryBodyRepresentationSingle::AtlasDocFormat
            }
        }
    }

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

    fn as_comment_body_write(self) -> generated_models::comment_body_write::Representation {
        match self {
            Self::Storage => generated_models::comment_body_write::Representation::Storage,
            Self::Wiki => generated_models::comment_body_write::Representation::Wiki,
            Self::AtlasDocFormat => {
                generated_models::comment_body_write::Representation::AtlasDocFormat
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

    fn into_update_blog_post_status(self) -> generated_models::update_blog_post_request::Status {
        match self {
            Self::Current => generated_models::update_blog_post_request::Status::Current,
            Self::Draft => generated_models::update_blog_post_request::Status::Draft,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluencePageCopy {
    pub source_id: String,
    pub title: String,
    pub space_id: Option<String>,
    pub parent_id: Option<String>,
    pub root_level: bool,
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
pub struct ConfluenceBlogPostUpdate {
    pub id: String,
    pub status: ConfluenceContentStatus,
    pub title: String,
    pub space_id: Option<String>,
    pub body: String,
    pub representation: ConfluenceBodyRepresentation,
    pub version: u64,
    pub message: Option<String>,
}

impl ConfluenceBlogPostUpdate {
    fn to_generated(&self) -> generated_models::UpdateBlogPostRequest {
        let mut version = generated_models::UpdateBlogPostRequestVersion::new();
        version.number = Some(self.version as i32);
        version.message.clone_from(&self.message);

        let mut request = generated_models::UpdateBlogPostRequest::new(
            self.id.clone(),
            self.status.into_update_blog_post_status(),
            self.title.clone(),
            generated_models::CreateBlogPostRequestBody::BlogPostBodyWrite(Box::new(
                generated_models::BlogPostBodyWrite {
                    representation: Some(self.representation.as_blog_post_body_write()),
                    value: Some(self.body.clone()),
                },
            )),
            version,
        );
        request.space_id.clone_from(&self.space_id);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceSpaceCreate {
    pub key: Option<String>,
    pub alias: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub private: bool,
}

impl ConfluenceSpaceCreate {
    fn to_generated(&self) -> generated_models::CreateSpaceRequest {
        let mut request = generated_models::CreateSpaceRequest::new(self.name.clone());
        request.key.clone_from(&self.key);
        request.alias.clone_from(&self.alias);
        request.create_private_space = self.private.then_some(true);
        request.description = self.description.as_ref().map(|description| {
            let mut generated = generated_models::CreateSpaceRequestDescription::new();
            generated.value = Some(description.clone());
            generated.representation = Some("plain".to_owned());
            Box::new(generated)
        });
        request
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceSpaceUpdate {
    pub key: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl ConfluenceSpaceUpdate {
    fn to_v1_update_request(&self) -> generated_v1_models::SpaceUpdate {
        let mut payload = generated_v1_models::SpaceUpdate::new();
        payload.name = self.name.clone().map(Some);
        if let Some(description) = &self.description {
            payload.description = Some(Some(Box::new(
                generated_v1_models::SpaceDescriptionCreate::new(
                    generated_v1_models::SpaceDescriptionCreatePlain {
                        value: Some(description.clone()),
                        representation: Some("plain".to_owned()),
                    },
                ),
            )));
        }
        payload
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceContentTreeSearch {
    pub page_id: String,
    pub limit: u32,
    pub depth: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceContentTreePage {
    #[serde(default)]
    pub results: Vec<ConfluenceContentNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceContentNode {
    pub id: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub space_id: Option<String>,
    pub parent_id: Option<String>,
    pub depth: Option<i32>,
    pub child_position: Option<i32>,
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
    pub body: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceAttachmentUpload {
    pub page_id: String,
    pub file: PathBuf,
    pub comment: Option<String>,
    pub minor_edit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceLabelSearch {
    pub content_id: String,
    pub prefix: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceLabelPage {
    #[serde(default)]
    pub results: Vec<ConfluenceLabel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceLabel {
    pub id: Option<String>,
    pub name: Option<String>,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceCommentSearch {
    pub content_id: String,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceCommentCreate {
    pub content_id: String,
    pub parent_comment_id: Option<String>,
    pub body: String,
    pub representation: ConfluenceBodyRepresentation,
}

impl ConfluenceCommentCreate {
    fn to_generated_page_footer(&self) -> generated_models::CreateFooterCommentModel {
        let mut comment = generated_models::CreateFooterCommentModel::new();
        comment.page_id = Some(self.content_id.clone());
        comment
            .parent_comment_id
            .clone_from(&self.parent_comment_id);
        comment.body = Some(Box::new(
            generated_models::CreateFooterCommentModelBody::CommentBodyWrite(Box::new(
                generated_models::CommentBodyWrite {
                    representation: Some(self.representation.as_comment_body_write()),
                    value: Some(self.body.clone()),
                },
            )),
        ));
        comment
    }

    fn to_generated_blog_footer(&self) -> generated_models::CreateFooterCommentModel {
        let mut comment = generated_models::CreateFooterCommentModel::new();
        comment.blog_post_id = Some(self.content_id.clone());
        comment
            .parent_comment_id
            .clone_from(&self.parent_comment_id);
        comment.body = Some(Box::new(
            generated_models::CreateFooterCommentModelBody::CommentBodyWrite(Box::new(
                generated_models::CommentBodyWrite {
                    representation: Some(self.representation.as_comment_body_write()),
                    value: Some(self.body.clone()),
                },
            )),
        ));
        comment
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceCommentPage {
    #[serde(default)]
    pub results: Vec<ConfluenceComment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceComment {
    pub id: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub page_id: Option<String>,
    pub blog_post_id: Option<String>,
    pub parent_comment_id: Option<String>,
    pub body: Option<String>,
    pub version: Option<ConfluenceVersion>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachmentDownload {
    pub attachment: ConfluenceAttachment,
    pub path: PathBuf,
    pub bytes: u64,
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
            body: crate::client::extract_api_error_body(&response.content),
        },
    }
}

fn generated_v1_error<T>(error: generated_v1_apis::Error<T>) -> ApiError {
    match error {
        generated_v1_apis::Error::Reqwest(error) => ApiError::Decode(error.to_string()),
        generated_v1_apis::Error::Serde(error) => ApiError::Decode(error.to_string()),
        generated_v1_apis::Error::Io(error) => ApiError::Decode(error.to_string()),
        generated_v1_apis::Error::ResponseError(response) => ApiError::Http {
            status: response.status,
            body: crate::client::extract_api_error_body(&response.content),
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

fn version_from_generated_v1(
    version: Option<Box<generated_v1_models::Version>>,
) -> Option<ConfluenceVersion> {
    version.map(|version| ConfluenceVersion {
        number: Some(version.number as u64),
        message: version.message,
        created_at: version.when.map(|created_at| created_at.to_rfc3339()),
    })
}

fn v1_link(
    links: &Option<std::collections::HashMap<String, String>>,
    name: &str,
) -> Option<String> {
    links.as_ref().and_then(|links| links.get(name).cloned())
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

fn body_from_generated_bulk(body: Option<Box<generated_models::BodyBulk>>) -> Option<String> {
    body.and_then(|body| {
        let body = *body;
        body.storage
            .and_then(|body| body.value)
            .or_else(|| body.atlas_doc_format.and_then(|body| body.value))
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

impl From<generated_models::CreateSpace201Response> for ConfluenceSpace {
    fn from(space: generated_models::CreateSpace201Response) -> Self {
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

impl From<generated_models::ChildrenResponse> for ConfluenceContentNode {
    fn from(child: generated_models::ChildrenResponse) -> Self {
        Self {
            id: child.id,
            status: child.status.map(|status| status.to_string()),
            title: child.title,
            content_type: child.r#type,
            space_id: child.space_id,
            parent_id: None,
            depth: None,
            child_position: child.child_position.flatten(),
        }
    }
}

impl From<generated_models::DescendantsResponse> for ConfluenceContentNode {
    fn from(descendant: generated_models::DescendantsResponse) -> Self {
        Self {
            id: descendant.id,
            status: descendant.status.map(|status| status.to_string()),
            title: descendant.title,
            content_type: descendant.r#type,
            space_id: None,
            parent_id: descendant.parent_id,
            depth: descendant.depth,
            child_position: descendant.child_position.flatten(),
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
            body: None,
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
            body: body_from_generated(post.body),
        }
    }
}

impl From<generated_models::MultiEntityResultLabel> for ConfluenceLabelPage {
    fn from(page: generated_models::MultiEntityResultLabel) -> Self {
        Self {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceLabel::from)
                .collect(),
        }
    }
}

impl From<generated_models::Label> for ConfluenceLabel {
    fn from(label: generated_models::Label) -> Self {
        Self {
            id: label.id,
            name: label.name,
            prefix: label.prefix,
        }
    }
}

impl From<generated_models::MultiEntityResultPageCommentModel> for ConfluenceCommentPage {
    fn from(page: generated_models::MultiEntityResultPageCommentModel) -> Self {
        Self {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceComment::from)
                .collect(),
        }
    }
}

impl From<generated_models::MultiEntityResultBlogPostCommentModel> for ConfluenceCommentPage {
    fn from(page: generated_models::MultiEntityResultBlogPostCommentModel) -> Self {
        Self {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceComment::from)
                .collect(),
        }
    }
}

impl From<generated_models::PageCommentModel> for ConfluenceComment {
    fn from(comment: generated_models::PageCommentModel) -> Self {
        Self {
            id: comment.id,
            status: comment.status.map(|status| status.to_string()),
            title: comment.title,
            page_id: comment.page_id,
            blog_post_id: None,
            parent_comment_id: None,
            body: body_from_generated_bulk(comment.body),
            version: version_from_generated(comment.version),
        }
    }
}

impl From<generated_models::BlogPostCommentModel> for ConfluenceComment {
    fn from(comment: generated_models::BlogPostCommentModel) -> Self {
        Self {
            id: comment.id,
            status: comment.status.map(|status| status.to_string()),
            title: comment.title,
            page_id: None,
            blog_post_id: comment.blog_post_id,
            parent_comment_id: None,
            body: body_from_generated_bulk(comment.body),
            version: version_from_generated(comment.version),
        }
    }
}

impl From<generated_models::CreateFooterComment201Response> for ConfluenceComment {
    fn from(comment: generated_models::CreateFooterComment201Response) -> Self {
        Self {
            id: comment.id,
            status: comment.status.map(|status| status.to_string()),
            title: comment.title,
            page_id: comment.page_id,
            blog_post_id: comment.blog_post_id,
            parent_comment_id: comment.parent_comment_id,
            body: body_from_generated(comment.body),
            version: version_from_generated(comment.version),
        }
    }
}

impl From<generated_v1_models::SearchPageResponseSearchResult> for ConfluenceSearchPage {
    fn from(page: generated_v1_models::SearchPageResponseSearchResult) -> Self {
        Self {
            results: page
                .results
                .into_iter()
                .map(ConfluenceSearchResult::from)
                .collect(),
            size: Some(page.size as u64),
            total_size: Some(page.total_size as u64),
        }
    }
}

impl From<generated_v1_models::SearchResult> for ConfluenceSearchResult {
    fn from(result: generated_v1_models::SearchResult) -> Self {
        Self {
            title: Some(result.title),
            url: Some(result.url),
            excerpt: Some(result.excerpt),
            content: result
                .content
                .map(|content| ConfluenceSearchContent::from(*content)),
        }
    }
}

impl From<generated_v1_models::Content> for ConfluenceSearchContent {
    fn from(content: generated_v1_models::Content) -> Self {
        Self {
            id: content.id,
            content_type: Some(content.r#type),
            status: Some(content.status),
            title: content.title,
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

impl From<generated_models::GetAttachmentById200Response> for ConfluenceAttachment {
    fn from(attachment: generated_models::GetAttachmentById200Response) -> Self {
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

impl From<generated_v1_models::Content> for ConfluenceAttachment {
    fn from(content: generated_v1_models::Content) -> Self {
        Self {
            id: content.id,
            status: Some(content.status),
            title: content.title,
            page_id: None,
            blog_post_id: None,
            media_type: None,
            media_type_description: None,
            file_id: None,
            file_size: None,
            webui_link: v1_link(&content._links, "webui"),
            download_link: v1_link(&content._links, "download"),
            version: version_from_generated_v1(content.version),
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
    fn builds_generated_create_space_request() {
        let request = ConfluenceSpaceCreate {
            key: Some("DEV".to_owned()),
            alias: None,
            name: "Development".to_owned(),
            description: Some("Team docs".to_owned()),
            private: true,
        }
        .to_generated();

        assert_eq!(request.name, "Development");
        assert_eq!(request.key.as_deref(), Some("DEV"));
        assert_eq!(request.create_private_space, Some(true));
        let description = request.description.expect("description");
        assert_eq!(description.value.as_deref(), Some("Team docs"));
        assert_eq!(description.representation.as_deref(), Some("plain"));
    }

    #[test]
    fn builds_v1_update_space_request() {
        let request = ConfluenceSpaceUpdate {
            key: "DEV".to_owned(),
            name: Some("Development".to_owned()),
            description: Some("Team docs".to_owned()),
        }
        .to_v1_update_request();

        assert_eq!(request.name, Some(Some("Development".to_owned())));
        let description = request
            .description
            .expect("description")
            .expect("description payload");
        assert_eq!(description.plain.value.as_deref(), Some("Team docs"));
        assert_eq!(description.plain.representation.as_deref(), Some("plain"));
    }

    #[test]
    fn converts_created_space_response() {
        let space = ConfluenceSpace::from(generated_models::CreateSpace201Response {
            id: Some("12345".to_owned()),
            key: Some("DEV".to_owned()),
            name: Some("Development".to_owned()),
            r#type: Some(generated_models::SpaceType::Global),
            status: Some(generated_models::SpaceStatus::Current),
            homepage_id: Some("67890".to_owned()),
            current_active_alias: Some("DEV".to_owned()),
            ..generated_models::CreateSpace201Response::new()
        });

        assert_eq!(space.key.as_deref(), Some("DEV"));
        assert_eq!(space.space_type.as_deref(), Some("global"));
        assert_eq!(space.status.as_deref(), Some("current"));
        assert_eq!(space.homepage_id.as_deref(), Some("67890"));
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
    fn converts_direct_child_content_node() {
        let child = ConfluenceContentNode::from(generated_models::ChildrenResponse {
            id: Some("333".to_owned()),
            status: None,
            title: Some("Folder".to_owned()),
            r#type: Some("folder".to_owned()),
            space_id: Some("12345".to_owned()),
            child_position: Some(Some(4)),
        });

        assert_eq!(child.id.as_deref(), Some("333"));
        assert_eq!(child.content_type.as_deref(), Some("folder"));
        assert_eq!(child.space_id.as_deref(), Some("12345"));
        assert_eq!(child.child_position, Some(4));
        assert_eq!(child.depth, None);
    }

    #[test]
    fn converts_descendant_content_node() {
        let descendant = ConfluenceContentNode::from(generated_models::DescendantsResponse {
            id: Some("444".to_owned()),
            status: None,
            title: Some("Whiteboard".to_owned()),
            r#type: Some("whiteboard".to_owned()),
            parent_id: Some("333".to_owned()),
            depth: Some(2),
            child_position: Some(Some(1)),
        });

        assert_eq!(descendant.id.as_deref(), Some("444"));
        assert_eq!(descendant.content_type.as_deref(), Some("whiteboard"));
        assert_eq!(descendant.parent_id.as_deref(), Some("333"));
        assert_eq!(descendant.depth, Some(2));
        assert_eq!(descendant.child_position, Some(1));
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
    fn parses_label_page() {
        let page: ConfluenceLabelPage = serde_json::from_str(
            r#"{
                "results": [
                    { "id": "1", "name": "runbook", "prefix": "global" }
                ]
            }"#,
        )
        .expect("parse labels");

        assert_eq!(page.results[0].name.as_deref(), Some("runbook"));
        assert_eq!(page.results[0].prefix.as_deref(), Some("global"));
    }

    #[test]
    fn builds_generated_footer_comment_request() {
        let comment = ConfluenceCommentCreate {
            content_id: "111".to_owned(),
            parent_comment_id: Some("222".to_owned()),
            body: "<p>Looks good</p>".to_owned(),
            representation: ConfluenceBodyRepresentation::Storage,
        };

        let generated = comment.to_generated_page_footer();

        assert_eq!(generated.page_id.as_deref(), Some("111"));
        assert_eq!(generated.parent_comment_id.as_deref(), Some("222"));
        let Some(body) = generated.body else {
            panic!("expected body");
        };
        let generated_models::CreateFooterCommentModelBody::CommentBodyWrite(body) = *body else {
            panic!("expected comment body write");
        };
        assert_eq!(body.value.as_deref(), Some("<p>Looks good</p>"));
    }

    #[test]
    fn builds_generated_blog_post_update_request() {
        let request = ConfluenceBlogPostUpdate {
            id: "222".to_owned(),
            status: ConfluenceContentStatus::Current,
            title: "Release Notes".to_owned(),
            space_id: Some("12345".to_owned()),
            body: "<p>Updated</p>".to_owned(),
            representation: ConfluenceBodyRepresentation::Storage,
            version: 3,
            message: Some("Update".to_owned()),
        }
        .to_generated();

        assert_eq!(request.id, "222");
        assert_eq!(request.title, "Release Notes");
        assert_eq!(request.space_id.as_deref(), Some("12345"));
        assert_eq!(request.version.number, Some(3));
        assert_eq!(request.version.message.as_deref(), Some("Update"));
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

    #[test]
    fn converts_v1_search_page() {
        let page = generated_v1_models::SearchPageResponseSearchResult::new(
            vec![generated_v1_models::SearchResult {
                title: "Runbook".to_owned(),
                excerpt: "Useful page".to_owned(),
                url: "/wiki/spaces/DEV/pages/111/Runbook".to_owned(),
                content: Some(Box::new(generated_v1_models::Content {
                    id: Some("111".to_owned()),
                    r#type: "page".to_owned(),
                    status: "current".to_owned(),
                    title: Some("Runbook".to_owned()),
                    version: None,
                    _links: None,
                })),
            }],
            1,
            1,
        );

        let page = ConfluenceSearchPage::from(page);
        let result = &page.results[0];
        assert_eq!(page.total_size, Some(1));
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
    fn converts_v1_attachment_upload_result() {
        let mut links = std::collections::HashMap::new();
        links.insert(
            "download".to_owned(),
            "/download/attachments/111/diagram.png".to_owned(),
        );

        let attachment = ConfluenceAttachment::from(generated_v1_models::Content {
            id: Some("att123".to_owned()),
            r#type: "attachment".to_owned(),
            status: "current".to_owned(),
            title: Some("diagram.png".to_owned()),
            version: Some(Box::new(generated_v1_models::Version::new(2))),
            _links: Some(links),
        });

        assert_eq!(attachment.id.as_deref(), Some("att123"));
        assert_eq!(attachment.title.as_deref(), Some("diagram.png"));
        assert_eq!(
            attachment.download_link.as_deref(),
            Some("/download/attachments/111/diagram.png")
        );
        assert_eq!(
            attachment
                .version
                .as_ref()
                .and_then(|version| version.number),
            Some(2)
        );
    }
}
