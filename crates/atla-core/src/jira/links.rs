use serde::Deserialize;

use super::JiraClient;
use super::models::{JiraIssueLink, JiraIssueLinkCreate, JiraLinkedIssue};
use super::util::generated_error;
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
            .map_err(generated_error)
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

    pub async fn delete_issue_link(&self, link_id: &str) -> Result<(), ApiError> {
        self.generated
            .delete_issue_link()
            .link_id(link_id)
            .send()
            .await
            .map(|_| ())
            .map_err(generated_error)
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
        let inward = link.inward_issue.unwrap();
        assert_eq!(inward.key.as_deref(), Some("PROJ-2"));
        assert_eq!(inward.summary.as_deref(), Some("Blocked task"));
        assert_eq!(inward.status.as_deref(), Some("Open"));
        let outward = link.outward_issue.unwrap();
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
