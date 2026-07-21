use atla_confluence_api::types as generated_types;
use atla_confluence_v1_api::types as generated_v1_types;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceBodyRepresentation {
    Storage,
    Wiki,
    AtlasDocFormat,
}

impl ConfluenceBodyRepresentation {
    pub(super) fn as_primary_body_single(self) -> generated_types::PrimaryBodyRepresentationSingle {
        match self {
            Self::Storage | Self::Wiki => generated_types::PrimaryBodyRepresentationSingle::Storage,
            Self::AtlasDocFormat => {
                generated_types::PrimaryBodyRepresentationSingle::AtlasDocFormat
            }
        }
    }

    pub(super) fn as_page_body_write(self) -> generated_types::PageBodyWriteRepresentation {
        match self {
            Self::Storage => generated_types::PageBodyWriteRepresentation::Storage,
            Self::Wiki => generated_types::PageBodyWriteRepresentation::Wiki,
            Self::AtlasDocFormat => generated_types::PageBodyWriteRepresentation::AtlasDocFormat,
        }
    }

    pub(super) fn as_blog_post_body_write(
        self,
    ) -> generated_types::BlogPostBodyWriteRepresentation {
        match self {
            Self::Storage => generated_types::BlogPostBodyWriteRepresentation::Storage,
            Self::Wiki => generated_types::BlogPostBodyWriteRepresentation::Wiki,
            Self::AtlasDocFormat => {
                generated_types::BlogPostBodyWriteRepresentation::AtlasDocFormat
            }
        }
    }

    pub(super) fn as_comment_body_write(self) -> generated_types::CommentBodyWriteRepresentation {
        match self {
            Self::Storage => generated_types::CommentBodyWriteRepresentation::Storage,
            Self::Wiki => generated_types::CommentBodyWriteRepresentation::Wiki,
            Self::AtlasDocFormat => generated_types::CommentBodyWriteRepresentation::AtlasDocFormat,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceContentStatus {
    Current,
    Draft,
}

impl ConfluenceContentStatus {
    pub(super) fn into_create_page_status(self) -> generated_types::CreatePageBodyStatus {
        match self {
            Self::Current => generated_types::CreatePageBodyStatus::Current,
            Self::Draft => generated_types::CreatePageBodyStatus::Draft,
        }
    }

    pub(super) fn into_update_page_status(self) -> generated_types::UpdatePageBodyStatus {
        match self {
            Self::Current => generated_types::UpdatePageBodyStatus::Current,
            Self::Draft => generated_types::UpdatePageBodyStatus::Draft,
        }
    }

    pub(super) fn into_update_page_title_status(
        self,
    ) -> generated_types::UpdatePageTitleBodyStatus {
        match self {
            Self::Current => generated_types::UpdatePageTitleBodyStatus::Current,
            Self::Draft => generated_types::UpdatePageTitleBodyStatus::Draft,
        }
    }

    pub(super) fn into_create_blog_post_status(self) -> generated_types::CreateBlogPostBodyStatus {
        match self {
            Self::Current => generated_types::CreateBlogPostBodyStatus::Current,
            Self::Draft => generated_types::CreateBlogPostBodyStatus::Draft,
        }
    }

    pub(super) fn into_update_blog_post_status(self) -> generated_types::UpdateBlogPostBodyStatus {
        match self {
            Self::Current => generated_types::UpdateBlogPostBodyStatus::Current,
            Self::Draft => generated_types::UpdateBlogPostBodyStatus::Draft,
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
    pub(super) fn to_generated(&self) -> generated_types::CreatePageBody {
        generated_types::CreatePageBody {
            body: self.body.as_ref().map(|body| {
                generated_types::PageBodyWrite {
                    representation: Some(self.representation.as_page_body_write()),
                    value: Some(body.clone()),
                }
                .into()
            }),
            parent_id: self.parent_id.clone(),
            space_id: self.space_id.clone(),
            status: Some(self.status.into_create_page_status()),
            subtype: None,
            title: Some(self.title.clone()),
        }
    }

    /// JSON request body `create_page` sends; used by --dry-run previews.
    pub fn request_body(&self) -> serde_json::Value {
        serde_json::to_value(self.to_generated()).expect("page create body is serializable")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluencePageTitleUpdate {
    pub title: String,
    pub status: ConfluenceContentStatus,
}

impl ConfluencePageTitleUpdate {
    pub(super) fn to_generated(&self) -> generated_types::UpdatePageTitleBody {
        generated_types::UpdatePageTitleBody {
            status: self.status.into_update_page_title_status(),
            title: self.title.clone(),
        }
    }

    /// JSON request body `update_page_title` sends; used by --dry-run previews.
    pub fn request_body(&self) -> serde_json::Value {
        serde_json::to_value(self.to_generated()).expect("page title update body is serializable")
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
    pub(super) fn to_generated(&self) -> generated_types::UpdatePageBody {
        generated_types::UpdatePageBody {
            body: generated_types::PageBodyWrite {
                representation: Some(self.representation.as_page_body_write()),
                value: Some(self.body.clone()),
            }
            .into(),
            id: self.id.clone(),
            owner_id: None,
            parent_id: self.parent_id.clone(),
            space_id: self.space_id.clone(),
            status: self.status.into_update_page_status(),
            title: self.title.clone(),
            version: generated_types::UpdatePageBodyVersion {
                message: self.message.clone(),
                number: Some(self.version as i32),
            },
        }
    }

    /// JSON request body `update_page` sends; used by --dry-run previews.
    pub fn request_body(&self) -> serde_json::Value {
        serde_json::to_value(self.to_generated()).expect("page update body is serializable")
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
    pub(super) fn to_generated(&self) -> generated_types::CreateBlogPostBody {
        generated_types::CreateBlogPostBody {
            body: self.body.as_ref().map(|body| {
                generated_types::BlogPostBodyWrite {
                    representation: Some(self.representation.as_blog_post_body_write()),
                    value: Some(body.clone()),
                }
                .into()
            }),
            created_at: None,
            space_id: self.space_id.clone(),
            status: Some(self.status.into_create_blog_post_status()),
            title: Some(self.title.clone()),
        }
    }

    /// JSON request body `create_blog_post` sends; used by --dry-run previews.
    pub fn request_body(&self) -> serde_json::Value {
        serde_json::to_value(self.to_generated()).expect("blog post create body is serializable")
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
    pub(super) fn to_generated(&self) -> generated_types::UpdateBlogPostBody {
        generated_types::UpdateBlogPostBody {
            body: generated_types::BlogPostBodyWrite {
                representation: Some(self.representation.as_blog_post_body_write()),
                value: Some(self.body.clone()),
            }
            .into(),
            created_at: None,
            id: self.id.clone(),
            space_id: self.space_id.clone(),
            status: self.status.into_update_blog_post_status(),
            title: self.title.clone(),
            version: generated_types::UpdateBlogPostBodyVersion {
                message: self.message.clone(),
                number: Some(self.version as i32),
            },
        }
    }

    /// JSON request body `update_blog_post` sends; used by --dry-run previews.
    pub fn request_body(&self) -> serde_json::Value {
        serde_json::to_value(self.to_generated()).expect("blog post update body is serializable")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceSpaceSearch {
    pub key: Option<String>,
    pub limit: u32,
    pub cursor: Option<String>,
}

impl Default for ConfluenceSpaceSearch {
    fn default() -> Self {
        Self {
            key: None,
            limit: 25,
            cursor: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSpacePage {
    #[serde(default)]
    pub results: Vec<ConfluenceSpace>,
    /// `true` when the server confirmed no more results follow. `false` when the
    /// caller's `limit` was reached but more results may exist server-side.
    /// `None` when the API did not report pagination state.
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_cursor: Option<String>,
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
    pub(super) fn to_generated(&self) -> generated_types::CreateSpaceBody {
        generated_types::CreateSpaceBody {
            alias: self.alias.clone(),
            copy_space_access_configuration: None,
            create_private_space: self.private.then_some(true),
            description: self.description.as_ref().map(|description| {
                generated_types::CreateSpaceBodyDescription {
                    representation: Some("plain".to_owned()),
                    value: Some(description.clone()),
                }
            }),
            key: self.key.clone(),
            name: self.name.clone(),
            role_assignments: Vec::new(),
            template_key: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceSpaceUpdate {
    pub key: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl ConfluenceSpaceUpdate {
    pub(super) fn to_v1_update_request(&self) -> generated_v1_types::SpaceUpdate {
        generated_v1_types::SpaceUpdate {
            description: self.description.as_ref().map(|description| {
                generated_v1_types::SpaceDescriptionCreate(Some(
                    generated_v1_types::SpaceDescriptionCreateInner {
                        plain: generated_v1_types::SpaceDescriptionCreateInnerPlain {
                            representation: Some("plain".to_owned()),
                            value: Some(description.clone()),
                        },
                    },
                ))
            }),
            homepage: None,
            name: self.name.clone().map(|name| {
                name.try_into()
                    .expect("validated Confluence space name should fit generated type")
            }),
            status: None,
            type_: None,
        }
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
    pub space_owner_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluencePageSearch {
    pub space_id: Option<String>,
    pub title: Option<String>,
    pub limit: u32,
    pub cursor: Option<String>,
}

impl Default for ConfluencePageSearch {
    fn default() -> Self {
        Self {
            space_id: None,
            title: None,
            limit: 25,
            cursor: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluencePagePage {
    #[serde(default)]
    pub results: Vec<ConfluencePage>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceContentTreeSearch {
    pub page_id: String,
    pub limit: u32,
    pub depth: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceContentTreePage {
    #[serde(default)]
    pub results: Vec<ConfluenceContentNode>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_cursor: Option<String>,
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
    pub cursor: Option<String>,
}

impl Default for ConfluenceBlogPostSearch {
    fn default() -> Self {
        Self {
            space_id: None,
            title: None,
            limit: 25,
            cursor: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceBlogPostPage {
    #[serde(default)]
    pub results: Vec<ConfluenceBlogPost>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_cursor: Option<String>,
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
    pub start: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSearchPage {
    #[serde(default)]
    pub results: Vec<ConfluenceSearchResult>,
    pub size: Option<u64>,
    pub total_size: Option<u64>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_start: Option<u32>,
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
    pub cursor: Option<String>,
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
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceLabelPage {
    #[serde(default)]
    pub results: Vec<ConfluenceLabel>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_cursor: Option<String>,
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
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceCommentCreate {
    pub content_id: String,
    pub parent_comment_id: Option<String>,
    pub body: String,
    pub representation: ConfluenceBodyRepresentation,
}

impl ConfluenceCommentCreate {
    pub(super) fn to_generated_page_footer(&self) -> generated_types::CreateFooterCommentModel {
        generated_types::CreateFooterCommentModel {
            attachment_id: None,
            blog_post_id: None,
            body: Some(
                generated_types::CommentBodyWrite {
                    representation: Some(self.representation.as_comment_body_write()),
                    value: Some(self.body.clone()),
                }
                .into(),
            ),
            custom_content_id: None,
            page_id: self
                .parent_comment_id
                .is_none()
                .then(|| self.content_id.clone()),
            parent_comment_id: self.parent_comment_id.clone(),
        }
    }

    pub(super) fn to_generated_blog_footer(&self) -> generated_types::CreateFooterCommentModel {
        generated_types::CreateFooterCommentModel {
            attachment_id: None,
            blog_post_id: self
                .parent_comment_id
                .is_none()
                .then(|| self.content_id.clone()),
            body: Some(
                generated_types::CommentBodyWrite {
                    representation: Some(self.representation.as_comment_body_write()),
                    value: Some(self.body.clone()),
                }
                .into(),
            ),
            custom_content_id: None,
            page_id: None,
            parent_comment_id: self.parent_comment_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceCommentPage {
    #[serde(default)]
    pub results: Vec<ConfluenceComment>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_cursor: Option<String>,
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
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_cursor: Option<String>,
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

fn to_value<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("generated Confluence types should serialize")
}

fn field<'a>(value: &'a Value, names: &[&str]) -> Option<&'a Value> {
    names.iter().find_map(|name| value.get(*name))
}

fn string_from_value(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .or_else(|| value.as_i64().map(|n| n.to_string()))
        .or_else(|| value.as_u64().map(|n| n.to_string()))
}

fn optional_string(value: &Value, names: &[&str]) -> Option<String> {
    field(value, names).and_then(string_from_value)
}

fn optional_i32(value: &Value, names: &[&str]) -> Option<i32> {
    field(value, names)
        .and_then(Value::as_i64)
        .and_then(|n| i32::try_from(n).ok())
}

fn optional_i64(value: &Value, names: &[&str]) -> Option<i64> {
    field(value, names).and_then(Value::as_i64)
}

fn optional_body_text(value: &Value) -> Option<String> {
    let body = field(value, &["body"])?;
    if let Some(text) = string_from_value(body) {
        return Some(text);
    }
    for key in [
        "storage",
        "atlas_doc_format",
        "atlasDocFormat",
        "view",
        "export_view",
        "exportView",
    ] {
        if let Some(text) = field(body, &[key]).and_then(|inner| optional_string(inner, &["value"]))
        {
            return Some(text);
        }
    }
    None
}

fn version_from_json(value: &Value) -> Option<ConfluenceVersion> {
    let version = field(value, &["version"])?;
    Some(ConfluenceVersion {
        number: optional_i32(version, &["number"]).map(|n| n as u64),
        message: optional_string(version, &["message"]),
        created_at: optional_string(version, &["createdAt", "when"]),
    })
}

fn content_node_from_json(value: &Value) -> ConfluenceContentNode {
    ConfluenceContentNode {
        id: optional_string(value, &["id"]),
        status: optional_string(value, &["status"]),
        title: optional_string(value, &["title"]),
        content_type: optional_string(value, &["type"]),
        space_id: optional_string(value, &["spaceId"]),
        parent_id: optional_string(value, &["parentId"]),
        depth: optional_i32(value, &["depth"]),
        child_position: optional_i32(value, &["childPosition"]),
    }
}

fn page_from_json(value: &Value) -> ConfluencePage {
    ConfluencePage {
        id: optional_string(value, &["id"]),
        status: optional_string(value, &["status"]),
        title: optional_string(value, &["title"]),
        space_id: optional_string(value, &["spaceId"]),
        parent_id: optional_string(value, &["parentId"]),
        author_id: optional_string(value, &["authorId"]),
        owner_id: optional_string(value, &["ownerId"]),
        created_at: optional_string(value, &["createdAt"]),
        version: version_from_json(value),
        body: optional_body_text(value),
    }
}

fn blog_post_from_json(value: &Value) -> ConfluenceBlogPost {
    ConfluenceBlogPost {
        id: optional_string(value, &["id"]),
        status: optional_string(value, &["status"]),
        title: optional_string(value, &["title"]),
        space_id: optional_string(value, &["spaceId"]),
        author_id: optional_string(value, &["authorId"]),
        created_at: optional_string(value, &["createdAt"]),
        version: version_from_json(value),
        body: optional_body_text(value),
    }
}

fn space_from_json(value: &Value) -> ConfluenceSpace {
    ConfluenceSpace {
        id: optional_string(value, &["id"]),
        key: optional_string(value, &["key"]),
        name: optional_string(value, &["name"]),
        space_type: optional_string(value, &["type"]),
        status: optional_string(value, &["status"]),
        homepage_id: optional_string(value, &["homepageId"]),
        current_active_alias: optional_string(value, &["currentActiveAlias"]),
        space_owner_id: optional_string(value, &["spaceOwnerId"]),
    }
}

fn label_from_json(value: &Value) -> ConfluenceLabel {
    ConfluenceLabel {
        id: optional_string(value, &["id"]),
        name: optional_string(value, &["name"]),
        prefix: optional_string(value, &["prefix"]),
    }
}

fn comment_from_json(value: &Value) -> ConfluenceComment {
    ConfluenceComment {
        id: optional_string(value, &["id"]),
        status: optional_string(value, &["status"]),
        title: optional_string(value, &["title"]),
        page_id: optional_string(value, &["pageId"]),
        blog_post_id: optional_string(value, &["blogPostId"]),
        parent_comment_id: optional_string(value, &["parentCommentId"]),
        body: optional_body_text(value),
        version: version_from_json(value),
    }
}

fn attachment_from_json(value: &Value) -> ConfluenceAttachment {
    ConfluenceAttachment {
        id: optional_string(value, &["id"]),
        status: optional_string(value, &["status"]),
        title: optional_string(value, &["title"]),
        page_id: optional_string(value, &["pageId"]),
        blog_post_id: optional_string(value, &["blogPostId"]),
        media_type: optional_string(value, &["mediaType"]),
        media_type_description: optional_string(value, &["mediaTypeDescription"]),
        file_id: optional_string(value, &["fileId"]),
        file_size: optional_i64(value, &["fileSize"]),
        webui_link: optional_string(value, &["webuiLink"]),
        download_link: optional_string(value, &["downloadLink"])
            .or_else(|| v1_link(value, "download")),
        version: version_from_json(value),
    }
}

fn v1_link(value: &Value, name: &str) -> Option<String> {
    field(value, &["_links", "links"])
        .and_then(|links| links.get(name))
        .and_then(string_from_value)
}

fn search_content_from_json(value: &Value) -> ConfluenceSearchContent {
    ConfluenceSearchContent {
        id: optional_string(value, &["id"]),
        content_type: optional_string(value, &["type"]),
        status: optional_string(value, &["status"]),
        title: optional_string(value, &["title"]),
    }
}

fn search_result_from_json(value: &Value) -> ConfluenceSearchResult {
    ConfluenceSearchResult {
        title: optional_string(value, &["title"]),
        url: optional_string(value, &["url"]),
        excerpt: optional_string(value, &["excerpt"]),
        content: field(value, &["content"]).map(search_content_from_json),
    }
}

impl From<generated_types::SpaceBulk> for ConfluenceSpace {
    fn from(space: generated_types::SpaceBulk) -> Self {
        space_from_json(&to_value(&space))
    }
}

impl From<generated_types::CreateSpaceResponse> for ConfluenceSpace {
    fn from(space: generated_types::CreateSpaceResponse) -> Self {
        space_from_json(&to_value(&space))
    }
}

impl From<generated_types::PageBulk> for ConfluencePage {
    fn from(page: generated_types::PageBulk) -> Self {
        page_from_json(&to_value(&page))
    }
}

impl From<generated_types::ChildrenResponse> for ConfluenceContentNode {
    fn from(child: generated_types::ChildrenResponse) -> Self {
        content_node_from_json(&to_value(&child))
    }
}

impl From<generated_types::DescendantsResponse> for ConfluenceContentNode {
    fn from(descendant: generated_types::DescendantsResponse) -> Self {
        content_node_from_json(&to_value(&descendant))
    }
}

impl From<generated_types::CreatePageResponse> for ConfluencePage {
    fn from(page: generated_types::CreatePageResponse) -> Self {
        page_from_json(&to_value(&page))
    }
}

impl From<generated_types::UpdatePageResponse> for ConfluencePage {
    fn from(page: generated_types::UpdatePageResponse) -> Self {
        page_from_json(&to_value(&page))
    }
}

impl From<generated_types::UpdatePageTitleResponse> for ConfluencePage {
    fn from(page: generated_types::UpdatePageTitleResponse) -> Self {
        page_from_json(&to_value(&page))
    }
}

impl From<generated_types::GetPageByIdResponse> for ConfluencePage {
    fn from(page: generated_types::GetPageByIdResponse) -> Self {
        page_from_json(&to_value(&page))
    }
}

impl From<generated_types::BlogPostBulk> for ConfluenceBlogPost {
    fn from(post: generated_types::BlogPostBulk) -> Self {
        blog_post_from_json(&to_value(&post))
    }
}

impl From<generated_types::CreateBlogPostResponse> for ConfluenceBlogPost {
    fn from(post: generated_types::CreateBlogPostResponse) -> Self {
        blog_post_from_json(&to_value(&post))
    }
}

impl From<generated_types::UpdateBlogPostResponse> for ConfluenceBlogPost {
    fn from(post: generated_types::UpdateBlogPostResponse) -> Self {
        blog_post_from_json(&to_value(&post))
    }
}

impl From<generated_types::GetBlogPostByIdResponse> for ConfluenceBlogPost {
    fn from(post: generated_types::GetBlogPostByIdResponse) -> Self {
        blog_post_from_json(&to_value(&post))
    }
}

impl From<generated_types::MultiEntityResultLabel> for ConfluenceLabelPage {
    fn from(page: generated_types::MultiEntityResultLabel) -> Self {
        let value = to_value(&page);
        Self {
            results: field(&value, &["results"])
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|label| label_from_json(&label))
                .collect(),
            is_last: None,
            next_cursor: None,
        }
    }
}

impl From<generated_types::Label> for ConfluenceLabel {
    fn from(label: generated_types::Label) -> Self {
        label_from_json(&to_value(&label))
    }
}

impl From<generated_v1_types::LabelArray> for ConfluenceLabelPage {
    fn from(page: generated_v1_types::LabelArray) -> Self {
        let value = to_value(&page);
        Self {
            results: field(&value, &["results"])
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|label| label_from_json(&label))
                .collect(),
            is_last: None,
            next_cursor: None,
        }
    }
}

impl From<generated_v1_types::Label> for ConfluenceLabel {
    fn from(label: generated_v1_types::Label) -> Self {
        label_from_json(&to_value(&label))
    }
}

impl From<generated_types::MultiEntityResultPageCommentModel> for ConfluenceCommentPage {
    fn from(page: generated_types::MultiEntityResultPageCommentModel) -> Self {
        let value = to_value(&page);
        Self {
            results: field(&value, &["results"])
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|comment| comment_from_json(&comment))
                .collect(),
            is_last: None,
            next_cursor: None,
        }
    }
}

impl From<generated_types::MultiEntityResultBlogPostCommentModel> for ConfluenceCommentPage {
    fn from(page: generated_types::MultiEntityResultBlogPostCommentModel) -> Self {
        let value = to_value(&page);
        Self {
            results: field(&value, &["results"])
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|comment| comment_from_json(&comment))
                .collect(),
            is_last: None,
            next_cursor: None,
        }
    }
}

impl From<generated_types::PageCommentModel> for ConfluenceComment {
    fn from(comment: generated_types::PageCommentModel) -> Self {
        comment_from_json(&to_value(&comment))
    }
}

impl From<generated_types::BlogPostCommentModel> for ConfluenceComment {
    fn from(comment: generated_types::BlogPostCommentModel) -> Self {
        comment_from_json(&to_value(&comment))
    }
}

impl From<generated_types::CreateFooterCommentResponse> for ConfluenceComment {
    fn from(comment: generated_types::CreateFooterCommentResponse) -> Self {
        comment_from_json(&to_value(&comment))
    }
}

impl From<generated_v1_types::SearchPageResponseSearchResult> for ConfluenceSearchPage {
    fn from(page: generated_v1_types::SearchPageResponseSearchResult) -> Self {
        let value = to_value(&page);
        Self {
            results: field(&value, &["results"])
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|result| search_result_from_json(&result))
                .collect(),
            size: optional_i32(&value, &["size"]).map(|n| n as u64),
            total_size: optional_i32(&value, &["totalSize"]).map(|n| n as u64),
            is_last: None,
            next_start: None,
        }
    }
}

impl From<generated_v1_types::SearchResult> for ConfluenceSearchResult {
    fn from(result: generated_v1_types::SearchResult) -> Self {
        search_result_from_json(&to_value(&result))
    }
}

impl From<generated_v1_types::Content> for ConfluenceSearchContent {
    fn from(content: generated_v1_types::Content) -> Self {
        search_content_from_json(&to_value(&content))
    }
}

impl From<generated_types::AttachmentBulk> for ConfluenceAttachment {
    fn from(attachment: generated_types::AttachmentBulk) -> Self {
        attachment_from_json(&to_value(&attachment))
    }
}

impl From<generated_types::GetAttachmentByIdResponse> for ConfluenceAttachment {
    fn from(attachment: generated_types::GetAttachmentByIdResponse) -> Self {
        attachment_from_json(&to_value(&attachment))
    }
}

impl From<generated_v1_types::Content> for ConfluenceAttachment {
    fn from(content: generated_v1_types::Content) -> Self {
        let value = to_value(&content);
        ConfluenceAttachment {
            id: optional_string(&value, &["id"]),
            status: optional_string(&value, &["status"]),
            title: optional_string(&value, &["title"]),
            page_id: None,
            blog_post_id: None,
            media_type: None,
            media_type_description: None,
            file_id: None,
            file_size: None,
            webui_link: v1_link(&value, "webui"),
            download_link: v1_link(&value, "download"),
            version: version_from_json(&value),
        }
    }
}
