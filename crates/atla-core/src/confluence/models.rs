use atla_confluence_api::models as generated_models;
use atla_confluence_v1_api::models as generated_v1_models;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceBodyRepresentation {
    Storage,
    Wiki,
    AtlasDocFormat,
}

impl ConfluenceBodyRepresentation {
    pub(super) fn as_primary_body_single(
        self,
    ) -> generated_models::PrimaryBodyRepresentationSingle {
        match self {
            Self::Storage | Self::Wiki => {
                generated_models::PrimaryBodyRepresentationSingle::Storage
            }
            Self::AtlasDocFormat => {
                generated_models::PrimaryBodyRepresentationSingle::AtlasDocFormat
            }
        }
    }

    pub(super) fn as_page_body_write(self) -> generated_models::page_body_write::Representation {
        match self {
            Self::Storage => generated_models::page_body_write::Representation::Storage,
            Self::Wiki => generated_models::page_body_write::Representation::Wiki,
            Self::AtlasDocFormat => {
                generated_models::page_body_write::Representation::AtlasDocFormat
            }
        }
    }

    pub(super) fn as_blog_post_body_write(
        self,
    ) -> generated_models::blog_post_body_write::Representation {
        match self {
            Self::Storage => generated_models::blog_post_body_write::Representation::Storage,
            Self::Wiki => generated_models::blog_post_body_write::Representation::Wiki,
            Self::AtlasDocFormat => {
                generated_models::blog_post_body_write::Representation::AtlasDocFormat
            }
        }
    }

    pub(super) fn as_comment_body_write(
        self,
    ) -> generated_models::comment_body_write::Representation {
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
    pub(super) fn into_create_page_status(self) -> generated_models::create_page_request::Status {
        match self {
            Self::Current => generated_models::create_page_request::Status::Current,
            Self::Draft => generated_models::create_page_request::Status::Draft,
        }
    }

    pub(super) fn into_update_page_status(self) -> generated_models::update_page_request::Status {
        match self {
            Self::Current => generated_models::update_page_request::Status::Current,
            Self::Draft => generated_models::update_page_request::Status::Draft,
        }
    }

    pub(super) fn into_update_page_title_status(
        self,
    ) -> generated_models::update_page_title_request::Status {
        match self {
            Self::Current => generated_models::update_page_title_request::Status::Current,
            Self::Draft => generated_models::update_page_title_request::Status::Draft,
        }
    }

    pub(super) fn into_create_blog_post_status(
        self,
    ) -> generated_models::create_blog_post_request::Status {
        match self {
            Self::Current => generated_models::create_blog_post_request::Status::Current,
            Self::Draft => generated_models::create_blog_post_request::Status::Draft,
        }
    }

    pub(super) fn into_update_blog_post_status(
        self,
    ) -> generated_models::update_blog_post_request::Status {
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
    pub(super) fn to_generated(&self) -> generated_models::CreatePageRequest {
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
    pub(super) fn to_generated(&self) -> generated_models::UpdatePageRequest {
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
    pub(super) fn to_generated(&self) -> generated_models::CreateBlogPostRequest {
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
    pub(super) fn to_generated(&self) -> generated_models::UpdateBlogPostRequest {
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
    pub(super) fn to_generated(&self) -> generated_models::CreateSpaceRequest {
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
    pub(super) fn to_v1_update_request(&self) -> generated_v1_models::SpaceUpdate {
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
    pub(super) fn to_generated_page_footer(&self) -> generated_models::CreateFooterCommentModel {
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

    pub(super) fn to_generated_blog_footer(&self) -> generated_models::CreateFooterCommentModel {
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

pub(super) fn version_from_generated(
    version: Option<Box<generated_models::Version>>,
) -> Option<ConfluenceVersion> {
    version.map(|version| ConfluenceVersion {
        number: version.number.map(|number| number as u64),
        message: version.message,
        created_at: version.created_at.map(|created_at| created_at.to_rfc3339()),
    })
}

pub(super) fn version_from_generated_v1(
    version: Option<Box<generated_v1_models::Version>>,
) -> Option<ConfluenceVersion> {
    version.map(|version| ConfluenceVersion {
        number: Some(version.number as u64),
        message: version.message,
        created_at: version.when.map(|created_at| created_at.to_rfc3339()),
    })
}

pub(super) fn v1_link(
    links: &Option<std::collections::HashMap<String, String>>,
    name: &str,
) -> Option<String> {
    links.as_ref().and_then(|links| links.get(name).cloned())
}

pub(super) fn body_from_generated(
    body: Option<Box<generated_models::BodySingle>>,
) -> Option<String> {
    body.and_then(|body| {
        let body = *body;
        body.storage
            .and_then(|body| body.value)
            .or_else(|| body.atlas_doc_format.and_then(|body| body.value))
            .or_else(|| body.view.and_then(|body| body.value))
    })
}

pub(super) fn body_from_generated_bulk(
    body: Option<Box<generated_models::BodyBulk>>,
) -> Option<String> {
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
