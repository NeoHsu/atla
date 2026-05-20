use super::JiraClient;
use super::models::{JiraBoard, JiraBoardPage, JiraBoardSearch};
use super::util::limit_i32;
use crate::client::{ApiError, read_json};

impl JiraClient {
    pub async fn search_boards(&self, search: &JiraBoardSearch) -> Result<JiraBoardPage, ApiError> {
        let mut request = self.raw_client.get("/rest/agile/1.0/board").query(&[
            ("startAt", search.start_at.min(i64::MAX as u64).to_string()),
            ("maxResults", limit_i32(search.max_results).to_string()),
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

        read_json(request).await
    }

    pub async fn get_board(&self, board_id: u64) -> Result<JiraBoard, ApiError> {
        read_json(
            self.raw_client
                .get(&format!("/rest/agile/1.0/board/{board_id}")),
        )
        .await
    }
}
