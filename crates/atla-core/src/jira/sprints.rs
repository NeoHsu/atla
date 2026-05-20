use chrono::Utc;

use super::JiraClient;
use super::models::{
    JiraIssueSearchPage, JiraSprint, JiraSprintCreate, JiraSprintPage, JiraSprintSearch,
    JiraSprintUpdate,
};
use super::util::{issue_fields, limit_i32};
use crate::client::{ApiError, read_empty, read_json};

impl JiraClient {
    pub async fn list_sprints(
        &self,
        search: &JiraSprintSearch,
    ) -> Result<JiraSprintPage, ApiError> {
        let mut request = self
            .raw_client
            .get(&format!("/rest/agile/1.0/board/{}/sprint", search.board_id))
            .query(&[
                ("startAt", search.start_at.min(i64::MAX as u64).to_string()),
                ("maxResults", limit_i32(search.max_results).to_string()),
            ]);
        if let Some(state) = &search.state {
            request = request.query(&[("state", state.as_str())]);
        }

        read_json(request).await
    }

    pub async fn get_sprint(&self, sprint_id: u64) -> Result<JiraSprint, ApiError> {
        read_json(
            self.raw_client
                .get(&format!("/rest/agile/1.0/sprint/{sprint_id}")),
        )
        .await
    }

    pub async fn get_sprint_issues(
        &self,
        sprint_id: u64,
        max_results: u32,
        fields: Option<Vec<String>>,
    ) -> Result<JiraIssueSearchPage, ApiError> {
        let fields_str = issue_fields(fields.as_deref()).join(",");
        read_json(
            self.raw_client
                .get(&format!("/rest/agile/1.0/sprint/{sprint_id}/issue"))
                .query(&[
                    ("maxResults", &limit_i32(max_results).to_string()),
                    ("fields", &fields_str),
                ]),
        )
        .await
    }

    pub async fn create_sprint(&self, sprint: &JiraSprintCreate) -> Result<JiraSprint, ApiError> {
        read_json(
            self.raw_client
                .post("/rest/agile/1.0/sprint")
                .json(&sprint.to_json()),
        )
        .await
    }

    pub async fn update_sprint(&self, update: &JiraSprintUpdate) -> Result<JiraSprint, ApiError> {
        let state_change = update.state.is_some();
        let needs_existing = update.name.is_none() || (state_change && update.start_date.is_none());

        let existing = if needs_existing {
            Some(self.get_sprint(update.id).await?)
        } else {
            None
        };

        let mut body = update.to_json();
        let obj = body.as_object_mut().expect("sprint update body is object");

        // Always include name (required by Jira API)
        let effective_name = update
            .name
            .clone()
            .or_else(|| existing.as_ref().and_then(|s| s.name.clone()))
            .unwrap_or_default();
        obj.entry("name").or_insert_with(|| effective_name.into());

        // On any state transition, Jira requires startDate (and endDate) in the body.
        if state_change && !obj.contains_key("startDate") {
            let start = update
                .start_date
                .clone()
                .or_else(|| existing.as_ref().and_then(|s| s.start_date.clone()))
                .unwrap_or_else(|| Utc::now().format("%Y-%m-%dT%H:%M:%S%.3f+0000").to_string());
            obj.insert("startDate".to_owned(), start.into());
        }
        if state_change && !obj.contains_key("endDate") {
            let end = update
                .end_date
                .clone()
                .or_else(|| existing.as_ref().and_then(|s| s.end_date.clone()))
                .unwrap_or_else(|| {
                    (Utc::now() + chrono::Duration::weeks(2))
                        .format("%Y-%m-%dT%H:%M:%S%.3f+0000")
                        .to_string()
                });
            obj.insert("endDate".to_owned(), end.into());
        }

        read_json(
            self.raw_client
                .post(&format!("/rest/agile/1.0/sprint/{}", update.id))
                .json(&body),
        )
        .await
    }

    pub async fn move_issues_to_sprint(
        &self,
        sprint_id: u64,
        issues: &[String],
    ) -> Result<(), ApiError> {
        read_empty(
            self.raw_client
                .post(&format!("/rest/agile/1.0/sprint/{sprint_id}/issue"))
                .json(&serde_json::json!({ "issues": issues })),
        )
        .await
    }

    pub async fn move_issues_to_backlog(&self, issues: &[String]) -> Result<(), ApiError> {
        read_empty(
            self.raw_client
                .post("/rest/agile/1.0/backlog/issue")
                .json(&serde_json::json!({ "issues": issues })),
        )
        .await
    }
}
