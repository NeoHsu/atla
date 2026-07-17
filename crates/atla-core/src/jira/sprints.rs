use chrono::Utc;

use super::JiraClient;
use super::models::{
    JiraIssue, JiraIssueSearchPage, JiraSprint, JiraSprintCreate, JiraSprintPage, JiraSprintSearch,
    JiraSprintUpdate,
};
use super::util::{JIRA_LIST_PAGE_CAP, issue_fields, limit_i32, next_offset};
use crate::client::{ApiError, read_empty, read_json};

impl JiraClient {
    pub async fn list_sprints(
        &self,
        search: &JiraSprintSearch,
    ) -> Result<JiraSprintPage, ApiError> {
        let max_results = self.raw_client.effective_item_limit(search.max_results);
        let mut collected: Vec<JiraSprint> = Vec::new();
        let mut start_at = search.start_at;
        let mut last_is_last: Option<bool> = Some(true);
        let mut last_total: Option<u64> = None;

        while self.raw_client.take_page() {
            let remaining = (max_results as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(JIRA_LIST_PAGE_CAP as u64) as u32;

            let mut request = self
                .raw_client
                .get(&format!("/rest/agile/1.0/board/{}/sprint", search.board_id))
                .query(&[
                    ("startAt", start_at.min(i64::MAX as u64).to_string()),
                    ("maxResults", limit_i32(page_size).to_string()),
                ]);
            if let Some(state) = &search.state {
                request = request.query(&[("state", state.as_str())]);
            }

            let page: JiraSprintPage = read_json(request).await?;
            let received = page.values.len() as u64;
            last_is_last = page.is_last;
            last_total = page.total;
            collected.extend(page.values);

            match next_offset(
                collected.len() as u64,
                max_results as u64,
                received,
                last_is_last,
                last_total,
                start_at,
            ) {
                Some(next) => start_at = next,
                None => break,
            }
        }

        let exhausted = matches!(last_is_last, Some(true))
            || last_total.is_some_and(|total| collected.len() as u64 >= total);
        if collected.len() > max_results as usize {
            collected.truncate(max_results as usize);
        }

        Ok(JiraSprintPage {
            start_at: search.start_at,
            max_results,
            total: last_total,
            is_last: Some(exhausted),
            values: collected,
        })
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
        self.get_sprint_issues_from(sprint_id, max_results, fields, 0)
            .await
    }

    pub async fn get_sprint_issues_from(
        &self,
        sprint_id: u64,
        max_results: u32,
        fields: Option<Vec<String>>,
        start_at: u64,
    ) -> Result<JiraIssueSearchPage, ApiError> {
        let fields_str = issue_fields(fields.as_deref()).join(",");
        let max_results = self.raw_client.effective_item_limit(max_results);
        let mut collected: Vec<JiraIssue> = Vec::new();
        let initial_start_at = start_at;
        let mut start_at: u64 = start_at;
        let mut last_total: Option<u64> = None;

        while self.raw_client.take_page() {
            let remaining = (max_results as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(JIRA_LIST_PAGE_CAP as u64) as u32;

            let page: SprintIssuesPage = read_json(
                self.raw_client
                    .get(&format!("/rest/agile/1.0/sprint/{sprint_id}/issue"))
                    .query(&[
                        ("startAt", &start_at.min(i64::MAX as u64).to_string()),
                        ("maxResults", &limit_i32(page_size).to_string()),
                        ("fields", &fields_str),
                    ]),
            )
            .await?;

            let received = page.issues.len() as u64;
            last_total = page.total;
            collected.extend(page.issues);

            match next_offset(
                collected.len() as u64,
                max_results as u64,
                received,
                None,
                last_total,
                start_at,
            ) {
                Some(next) => start_at = next,
                None => break,
            }
        }

        let next_start = initial_start_at + collected.len() as u64;
        let exhausted = last_total.is_some_and(|total| next_start >= total);
        if collected.len() > max_results as usize {
            collected.truncate(max_results as usize);
        }

        Ok(JiraIssueSearchPage {
            is_last: Some(exhausted),
            next_page_token: if exhausted {
                None
            } else {
                Some(next_start.to_string())
            },
            issues: collected,
        })
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

/// Wire-level shape returned by `/rest/agile/1.0/sprint/{id}/issue`. Captures
/// `total` so the paginator can detect exhaustion (this endpoint does not emit
/// `isLast`).
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SprintIssuesPage {
    #[serde(default)]
    total: Option<u64>,
    #[serde(default)]
    issues: Vec<JiraIssue>,
}
