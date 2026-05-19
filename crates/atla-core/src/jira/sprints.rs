use atla_jira_api::apis as generated_apis;
use chrono::Utc;

use super::JiraClient;
use super::models::{
    JiraIssueSearchPage, JiraSprint, JiraSprintCreate, JiraSprintPage, JiraSprintSearch,
    JiraSprintUpdate,
};
use super::util::{generated_error, issue_fields, limit_i32};
use crate::client::ApiError;

impl JiraClient {
    pub async fn list_sprints(
        &self,
        search: &JiraSprintSearch,
    ) -> Result<JiraSprintPage, ApiError> {
        let value = generated_apis::agile_sprints_api::get_sprints_for_board(
            &self.generated,
            search.board_id as i64,
            search.start_at.min(i64::MAX as u64) as i64,
            limit_i32(search.max_results),
            search.state.as_deref(),
        )
        .await
        .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    pub async fn get_sprint(&self, sprint_id: u64) -> Result<JiraSprint, ApiError> {
        let value =
            generated_apis::agile_sprints_api::get_sprint(&self.generated, sprint_id as i64)
                .await
                .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    pub async fn get_sprint_issues(
        &self,
        sprint_id: u64,
        max_results: u32,
        fields: Option<Vec<String>>,
    ) -> Result<JiraIssueSearchPage, ApiError> {
        let fields_str = issue_fields(fields.as_deref());
        let value = generated_apis::agile_sprints_api::get_issues_for_sprint(
            &self.generated,
            sprint_id as i64,
            limit_i32(max_results),
            &fields_str.join(","),
        )
        .await
        .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    pub async fn create_sprint(&self, sprint: &JiraSprintCreate) -> Result<JiraSprint, ApiError> {
        let value =
            generated_apis::agile_sprints_api::create_sprint(&self.generated, sprint.to_json())
                .await
                .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
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
        // Include endDate on any state transition (required by Jira API); preserve or default.
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

        let value = generated_apis::agile_sprints_api::update_sprint(
            &self.generated,
            update.id as i64,
            body,
        )
        .await
        .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    pub async fn move_issues_to_sprint(
        &self,
        sprint_id: u64,
        issues: &[String],
    ) -> Result<(), ApiError> {
        generated_apis::agile_sprints_api::move_issues_to_sprint(
            &self.generated,
            sprint_id as i64,
            serde_json::json!({ "issues": issues }),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn move_issues_to_backlog(&self, issues: &[String]) -> Result<(), ApiError> {
        generated_apis::agile_sprints_api::move_issues_to_backlog(
            &self.generated,
            serde_json::json!({ "issues": issues }),
        )
        .await
        .map_err(generated_error)
    }
}
