use atla_jira_api::apis as generated_apis;

use super::JiraClient;
use super::models::{JiraBoard, JiraBoardPage, JiraBoardSearch};
use super::util::{generated_error, limit_i32};
use crate::client::ApiError;

impl JiraClient {
    pub async fn search_boards(&self, search: &JiraBoardSearch) -> Result<JiraBoardPage, ApiError> {
        let value = generated_apis::agile_boards_api::search_boards(
            &self.generated,
            search.start_at.min(i64::MAX as u64) as i64,
            limit_i32(search.max_results),
            search.board_type.as_deref(),
            search.name.as_deref(),
            search.project_key_or_id.as_deref(),
        )
        .await
        .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    pub async fn get_board(&self, board_id: u64) -> Result<JiraBoard, ApiError> {
        let value = generated_apis::agile_boards_api::get_board(&self.generated, board_id as i64)
            .await
            .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }
}
