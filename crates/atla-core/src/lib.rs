pub mod auth;
pub mod client;
pub mod confluence;
pub mod jira;
pub mod markdown;
pub mod profile;

pub use auth::{AuthState, CredentialRef, KeyringCredentialStore};
pub use client::{AtlassianClient, AtlassianInstance};
pub use confluence::{
    ConfluenceClient, ConfluencePage, ConfluencePagePage, ConfluencePageSearch, ConfluenceSpace,
    ConfluenceSpacePage, ConfluenceSpaceSearch,
};
pub use jira::{
    JiraClient, JiraIssue, JiraIssueSearch, JiraIssueSearchPage, JiraProject, JiraProjectPage,
    JiraProjectSearch,
};
pub use profile::{AtlaConfig, ConfigStore, Profile};
