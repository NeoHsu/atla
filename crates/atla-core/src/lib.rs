pub mod auth;
pub mod client;
pub mod confluence;
pub mod jira;
pub mod markdown;
pub mod profile;

pub use auth::{AuthState, CredentialRef, KeyringCredentialStore};
pub use client::{AtlassianClient, AtlassianInstance};
pub use confluence::{
    ConfluenceAttachment, ConfluenceAttachmentPage, ConfluenceAttachmentSearch,
    ConfluenceAttachmentUpload, ConfluenceBlogPost, ConfluenceBlogPostCreate,
    ConfluenceBlogPostPage, ConfluenceBlogPostSearch, ConfluenceBodyRepresentation,
    ConfluenceClient, ConfluenceContentStatus, ConfluencePage, ConfluencePageCreate,
    ConfluencePagePage, ConfluencePageSearch, ConfluencePageUpdate, ConfluenceSearch,
    ConfluenceSearchPage, ConfluenceSearchResult, ConfluenceSpace, ConfluenceSpacePage,
    ConfluenceSpaceSearch,
};
pub use jira::{
    JiraClient, JiraComment, JiraCommentPage, JiraCreatedIssue, JiraIssue, JiraIssueCreate,
    JiraIssueList, JiraIssueSearch, JiraIssueSearchPage, JiraIssueUpdate, JiraProject,
    JiraProjectPage, JiraProjectSearch, JiraStatus, JiraTransition,
};
pub use profile::{AtlaConfig, ConfigStore, Profile};
