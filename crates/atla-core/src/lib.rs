pub mod auth;
pub mod client;
pub mod confluence;
pub(crate) mod generated_api;
pub mod http_policy;
pub mod jira;
pub mod markdown;
pub mod profile;
pub mod secure_file;

pub use auth::{
    AuthState, CredentialRef, CredentialStorage, FileCredentialStore, KeyringCredentialStore,
    TenantDiscovery, discover_tenant,
};
pub use client::{AtlassianClient, AtlassianInstance};
pub use confluence::{
    ConfluenceAttachment, ConfluenceAttachmentDownload, ConfluenceAttachmentPage,
    ConfluenceAttachmentSearch, ConfluenceAttachmentUpload, ConfluenceBlogPost,
    ConfluenceBlogPostCreate, ConfluenceBlogPostPage, ConfluenceBlogPostSearch,
    ConfluenceBlogPostUpdate, ConfluenceBodyRepresentation, ConfluenceClient, ConfluenceComment,
    ConfluenceCommentCreate, ConfluenceCommentPage, ConfluenceCommentSearch, ConfluenceContentNode,
    ConfluenceContentStatus, ConfluenceContentTreePage, ConfluenceContentTreeSearch,
    ConfluenceLabel, ConfluenceLabelPage, ConfluenceLabelSearch, ConfluencePage,
    ConfluencePageCopy, ConfluencePageCreate, ConfluencePagePage, ConfluencePageSearch,
    ConfluencePageTitleUpdate, ConfluencePageUpdate, ConfluenceSearch, ConfluenceSearchContent,
    ConfluenceSearchPage, ConfluenceSearchResult, ConfluenceSpace, ConfluenceSpaceCreate,
    ConfluenceSpacePage, ConfluenceSpaceSearch, ConfluenceSpaceUpdate, ConfluenceVersion,
};
pub use http_policy::HttpPolicy;
pub use jira::{
    JiraAssigneeTarget, JiraAttachment, JiraAttachmentDownload, JiraBoard, JiraBoardPage,
    JiraBoardSearch, JiraClient, JiraComment, JiraCommentPage, JiraCreatedIssue,
    JiraFieldAllowedValue, JiraGithubCommit, JiraGithubPullRequest, JiraIssue, JiraIssueAssign,
    JiraIssueCreate, JiraIssueField, JiraIssueFieldsQuery, JiraIssueLabelUpdate, JiraIssueLink,
    JiraIssueLinkCreate, JiraIssueList, JiraIssueSearch, JiraIssueSearchPage, JiraIssueType,
    JiraIssueUpdate, JiraLinkedIssue, JiraProject, JiraProjectPage, JiraProjectSearch, JiraSprint,
    JiraSprintCreate, JiraSprintPage, JiraSprintSearch, JiraSprintUpdate, JiraStatus,
    JiraTransition, JiraUser, JiraWorklog, JiraWorklogCreate, JiraWorklogPage,
    default_issue_fields,
};
pub use profile::{
    AtlaConfig, AtlassianProduct, ConfigStore, PolicyDecisionSource, PolicyMode, Profile,
    ProfilePolicy, ProfilePolicyDecision,
};
