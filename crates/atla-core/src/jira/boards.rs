use super::JiraClient;
use super::models::{JiraBoard, JiraBoardPage, JiraBoardSearch};
use super::util::{JIRA_LIST_PAGE_CAP, limit_i32, next_offset};
use crate::client::{ApiError, read_json};

impl JiraClient {
    pub async fn search_boards(&self, search: &JiraBoardSearch) -> Result<JiraBoardPage, ApiError> {
        let max_results = self.raw_client.effective_item_limit(search.max_results);
        let mut collected: Vec<JiraBoard> = Vec::new();
        let mut start_at = search.start_at;
        let mut last_is_last: Option<bool> = Some(true);
        let mut last_total: Option<u64> = None;

        while self.raw_client.take_page() {
            let remaining = (max_results as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(JIRA_LIST_PAGE_CAP as u64) as u32;

            let mut request = self.raw_client.get("/rest/agile/1.0/board").query(&[
                ("startAt", start_at.min(i64::MAX as u64).to_string()),
                ("maxResults", limit_i32(page_size).to_string()),
            ]);
            if let Some(board_type) = &search.board_type {
                request = request.query(&[("type", board_type.as_str())]);
            }
            if let Some(name) = &search.name {
                request = request.query(&[("name", name.as_str())]);
            }
            if let Some(project) = &search.project_key_or_id {
                request = request.query(&[("projectKeyOrId", project.as_str())]);
            }

            let page: JiraBoardPage = read_json(request).await?;
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

        Ok(JiraBoardPage {
            start_at: search.start_at,
            max_results,
            total: last_total,
            is_last: Some(exhausted),
            values: collected,
        })
    }

    pub async fn get_board(&self, board_id: u64) -> Result<JiraBoard, ApiError> {
        read_json(
            self.raw_client
                .get(&format!("/rest/agile/1.0/board/{board_id}")),
        )
        .await
    }
}
