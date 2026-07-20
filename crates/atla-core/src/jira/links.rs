use serde::Deserialize;

use super::JiraClient;

/// Allowlist of known Jira dev-status applicationType keys that back GitHub data.
/// Includes both Atlassian-native GitHub apps and third-party integrations
/// (e.g. GitKraken / BigBrassBand) that can manage GitHub repositories.
/// Keys are stored lowercase; comparisons are case-insensitive.
const GITHUB_APP_TYPE_KEYS: &[&str] = &[
    "github",
    "github enterprise",
    // GitKraken / Git Integration for Jira (BigBrassBand)
    "oauth-com.xiplink.jira.git.jira_git_plugin",
];
use super::models::{
    JiraGithubCommit, JiraGithubPullRequest, JiraIssueLink, JiraIssueLinkCreate, JiraLinkedIssue,
};
use super::util::ProgenitorResultExt;
use crate::client::{ApiError, read_json};

impl JiraClient {
    pub async fn create_issue_link(&self, link: &JiraIssueLinkCreate) -> Result<(), ApiError> {
        let resolved = JiraIssueLinkCreate {
            link_type: self.resolve_link_type(&link.link_type).await?,
            ..link.clone()
        };

        self.generated
            .link_issues()
            .body(resolved.to_generated())
            .send()
            .await
            .map(|_| ())
            .or_api_error()
            .await
    }

    pub async fn list_issue_links(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraIssueLink>, ApiError> {
        let issue = self
            .get_issue(issue_id_or_key, Some(vec!["issuelinks".to_owned()]))
            .await?;

        Ok(issue
            .fields
            .get("issuelinks")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .map(jira_issue_link_from_value)
            .collect())
    }

    pub async fn list_github_pull_requests(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraGithubPullRequest>, ApiError> {
        let issue_id = self.resolve_issue_numeric_id(issue_id_or_key).await?;
        let app_types = self.dev_status_pr_app_types(&issue_id).await?;

        // Issue 6: only query known GitHub-backed integrations (allowlist)
        let github_types: Vec<String> = app_types
            .into_iter()
            .filter(|t| {
                let lower = t.to_ascii_lowercase();
                GITHUB_APP_TYPE_KEYS.iter().any(|k| lower == *k)
            })
            .collect();

        // Issue 4: no linked GitHub PRs — return empty instead of guessing
        if github_types.is_empty() {
            return Ok(vec![]);
        }

        let mut prs = Vec::new();
        for app_type in github_types {
            // Issue 1: percent-encode the applicationType value
            let encoded = percent_encode_query_value(&app_type);
            let path = format!(
                "/rest/dev-status/1.0/issue/detail?issueId={issue_id}&applicationType={encoded}&dataType=pullrequest"
            );
            let response: DevStatusDetailResponse = read_json(self.raw_client.get(&path)).await?;
            prs.extend(
                response
                    .detail
                    .into_iter()
                    .flat_map(|repo| repo.pull_requests)
                    .map(|pr| JiraGithubPullRequest {
                        id: pr.id,
                        title: pr.title,
                        url: pr.url,
                        status: pr.status,
                        author: pr.author.and_then(|a| a.name),
                        source_branch: pr.source.and_then(|s| s.branch_name()),
                        destination_branch: pr.destination.and_then(|d| d.branch_name()),
                    }),
            );
        }
        Ok(prs)
    }

    pub async fn list_github_commits(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraGithubCommit>, ApiError> {
        let issue_id = self.resolve_issue_numeric_id(issue_id_or_key).await?;
        let app_types = self.dev_status_repo_app_types(&issue_id).await?;

        // Issue 6: only query known GitHub-backed integrations (allowlist)
        let github_types: Vec<String> = app_types
            .into_iter()
            .filter(|t| {
                let lower = t.to_ascii_lowercase();
                GITHUB_APP_TYPE_KEYS.iter().any(|k| lower == *k)
            })
            .collect();

        // Issue 4: no linked GitHub commits — return empty instead of guessing
        if github_types.is_empty() {
            return Ok(vec![]);
        }

        let mut commits = Vec::new();
        for app_type in github_types {
            // Issue 1: percent-encode the applicationType value
            let encoded = percent_encode_query_value(&app_type);
            let path = format!(
                "/rest/dev-status/1.0/issue/detail?issueId={issue_id}&applicationType={encoded}&dataType=repository"
            );
            let response: DevStatusRepoDetailResponse =
                read_json(self.raw_client.get(&path)).await?;
            for repo_group in response.detail {
                for repo in repo_group.repositories {
                    for commit in repo.commits {
                        commits.push(JiraGithubCommit {
                            id: commit.display_id,
                            message: commit.message,
                            url: commit.url,
                            author: commit.author.and_then(|a| a.name),
                            repository: repo.name.clone(),
                            repository_url: repo.url.clone(),
                            timestamp: commit.author_timestamp,
                        });
                    }
                }
            }
        }
        Ok(commits)
    }

    async fn resolve_issue_numeric_id(&self, issue_id_or_key: &str) -> Result<String, ApiError> {
        let issue = self
            .get_issue(issue_id_or_key, Some(vec!["id".to_owned()]))
            .await?;
        issue
            .id
            .ok_or_else(|| ApiError::Decode(format!("issue `{issue_id_or_key}` has no numeric id")))
    }

    async fn dev_status_summary(&self, issue_id: &str) -> Result<DevStatusSummary, ApiError> {
        let path = format!("/rest/dev-status/1.0/issue/summary?issueId={issue_id}");
        read_json(self.raw_client.get(&path)).await
    }

    async fn dev_status_pr_app_types(&self, issue_id: &str) -> Result<Vec<String>, ApiError> {
        let summary = self.dev_status_summary(issue_id).await?;
        Ok(summary
            .summary
            .pullrequest
            .map(|pr| pr.by_instance_type.into_keys().collect())
            .unwrap_or_default())
    }

    async fn dev_status_repo_app_types(&self, issue_id: &str) -> Result<Vec<String>, ApiError> {
        let summary = self.dev_status_summary(issue_id).await?;
        Ok(summary
            .summary
            .repository
            .map(|r| r.by_instance_type.into_keys().collect())
            .unwrap_or_default())
    }

    pub async fn delete_issue_link(&self, link_id: &str) -> Result<(), ApiError> {
        self.generated
            .delete_issue_link()
            .link_id(link_id)
            .send()
            .await
            .map(|_| ())
            .or_api_error()
            .await
    }

    async fn resolve_link_type(&self, user_input: &str) -> Result<String, ApiError> {
        let response: JiraIssueLinkTypesResponse =
            read_json(self.raw_client.get("/rest/api/3/issueLinkType")).await?;
        Ok(
            canonical_link_type_name(user_input, &response.issue_link_types)
                .unwrap_or(user_input)
                .to_owned(),
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevStatusSummary {
    summary: DevStatusSummaryBody,
}

#[derive(Debug, Deserialize)]
struct DevStatusSummaryBody {
    pullrequest: Option<DevStatusSummarySection>,
    repository: Option<DevStatusSummarySection>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevStatusSummarySection {
    #[serde(default)]
    by_instance_type: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevStatusDetailResponse {
    #[serde(default)]
    detail: Vec<DevStatusPrGroup>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevStatusPrGroup {
    #[serde(default)]
    pull_requests: Vec<DevStatusPr>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevStatusRepoDetailResponse {
    #[serde(default)]
    detail: Vec<DevStatusRepoGroup>,
}

#[derive(Debug, Deserialize)]
struct DevStatusRepoGroup {
    #[serde(default)]
    repositories: Vec<DevStatusRepository>,
}

#[derive(Debug, Deserialize)]
struct DevStatusRepository {
    name: Option<String>,
    url: Option<String>,
    #[serde(default)]
    commits: Vec<DevStatusCommit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevStatusCommit {
    display_id: Option<String>,
    message: Option<String>,
    url: Option<String>,
    author: Option<DevStatusAuthor>,
    author_timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevStatusPr {
    id: Option<String>,
    title: Option<String>,
    url: Option<String>,
    status: Option<String>,
    author: Option<DevStatusAuthor>,
    source: Option<DevStatusRef>,
    destination: Option<DevStatusRef>,
}

#[derive(Debug, Deserialize)]
struct DevStatusAuthor {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DevStatusRef {
    branch: Option<serde_json::Value>,
}

impl DevStatusRef {
    fn branch_name(self) -> Option<String> {
        match self.branch? {
            serde_json::Value::String(s) => Some(s),
            serde_json::Value::Object(obj) => {
                obj.get("name").and_then(|v| v.as_str()).map(str::to_owned)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JiraIssueLinkTypesResponse {
    #[serde(default)]
    issue_link_types: Vec<JiraIssueLinkTypeValue>,
}

#[derive(Debug, Deserialize)]
struct JiraIssueLinkTypeValue {
    name: String,
    #[serde(default)]
    inward: String,
    #[serde(default)]
    outward: String,
}

fn canonical_link_type_name<'a>(
    user_input: &str,
    link_types: &'a [JiraIssueLinkTypeValue],
) -> Option<&'a str> {
    link_types
        .iter()
        .find(|link_type| {
            link_type.name.eq_ignore_ascii_case(user_input)
                || link_type.inward.eq_ignore_ascii_case(user_input)
                || link_type.outward.eq_ignore_ascii_case(user_input)
        })
        .map(|link_type| link_type.name.as_str())
}

fn jira_issue_link_from_value(value: &serde_json::Value) -> JiraIssueLink {
    JiraIssueLink {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        link_type: value
            .get("type")
            .and_then(|link_type| link_type.get("name"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        inward_issue: value.get("inwardIssue").map(jira_linked_issue_from_value),
        outward_issue: value.get("outwardIssue").map(jira_linked_issue_from_value),
    }
}

fn jira_linked_issue_from_value(value: &serde_json::Value) -> JiraLinkedIssue {
    JiraLinkedIssue {
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        key: value
            .get("key")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        summary: value
            .get("fields")
            .and_then(|fields| fields.get("summary"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        status: value
            .get("fields")
            .and_then(|fields| fields.get("status"))
            .and_then(|status| status.get("name"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
    }
}

fn percent_encode_query_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                out.push('%');
                out.push(HEX[usize::from(byte >> 4)] as char);
                out.push(HEX[usize::from(byte & 0x0f)] as char);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_link_from_value() {
        let value = serde_json::json!({
            "id": "link-1",
            "type": { "name": "Blocks" },
            "inwardIssue": {
                "id": "10001",
                "key": "PROJ-2",
                "fields": {
                    "summary": "Blocked task",
                    "status": { "name": "Open" }
                }
            },
            "outwardIssue": {
                "id": "10002",
                "key": "PROJ-3",
                "fields": {
                    "summary": "Downstream",
                    "status": { "name": "Done" }
                }
            }
        });

        let link = jira_issue_link_from_value(&value);

        assert_eq!(link.id.as_deref(), Some("link-1"));
        assert_eq!(link.link_type.as_deref(), Some("Blocks"));
        let inward = link.inward_issue.expect("inward issue");
        assert_eq!(inward.key.as_deref(), Some("PROJ-2"));
        assert_eq!(inward.summary.as_deref(), Some("Blocked task"));
        assert_eq!(inward.status.as_deref(), Some("Open"));
        let outward = link.outward_issue.expect("outward issue");
        assert_eq!(outward.key.as_deref(), Some("PROJ-3"));
        assert_eq!(outward.status.as_deref(), Some("Done"));
    }

    #[test]
    fn parses_linked_issue_summary_and_status() {
        let value = serde_json::json!({
            "id": "10010",
            "key": "ABC-5",
            "fields": {
                "summary": "Some task",
                "status": { "name": "In Progress" }
            }
        });

        let linked = jira_linked_issue_from_value(&value);

        assert_eq!(linked.id.as_deref(), Some("10010"));
        assert_eq!(linked.key.as_deref(), Some("ABC-5"));
        assert_eq!(linked.summary.as_deref(), Some("Some task"));
        assert_eq!(linked.status.as_deref(), Some("In Progress"));
    }

    #[test]
    fn percent_encodes_query_values_without_panicking() {
        assert_eq!(
            percent_encode_query_value("github.com / ✓"),
            "github.com%20%2F%20%E2%9C%93"
        );
    }

    #[test]
    fn resolves_link_type_case_insensitively() {
        let link_types = vec![
            JiraIssueLinkTypeValue {
                name: "Blocks".to_owned(),
                inward: "is blocked by".to_owned(),
                outward: "blocks".to_owned(),
            },
            JiraIssueLinkTypeValue {
                name: "Relates".to_owned(),
                inward: "relates to".to_owned(),
                outward: "relates to".to_owned(),
            },
        ];

        assert_eq!(
            canonical_link_type_name("blocks", &link_types),
            Some("Blocks")
        );
        assert_eq!(
            canonical_link_type_name("IS BLOCKED BY", &link_types),
            Some("Blocks")
        );
        assert_eq!(
            canonical_link_type_name("relates", &link_types),
            Some("Relates")
        );
        assert_eq!(canonical_link_type_name("duplicates", &link_types), None);
    }
}
