use atla_jira_api::types as generated_types;

use super::JiraClient;
use super::models::{JiraComment, JiraCommentPage};
use super::util::{JIRA_LIST_PAGE_CAP, ProgenitorResultExt, adf_body, limit_i32, next_offset};
use crate::client::ApiError;

/// JSON request body `add_comment`/`update_comment` send (Markdown converted
/// to ADF); used by --dry-run previews.
pub fn comment_request_body(body: &str) -> serde_json::Value {
    serde_json::json!({ "body": adf_body(body) })
}

impl JiraClient {
    pub async fn add_comment(
        &self,
        issue_id_or_key: &str,
        body: &str,
    ) -> Result<JiraComment, ApiError> {
        let comment = self
            .generated
            .add_comment()
            .issue_id_or_key(issue_id_or_key)
            .body(generated_types::CommentCreateRequest {
                body: adf_body(body),
            })
            .send()
            .await
            .or_api_error()
            .await?;

        Ok(comment.into_inner().into())
    }

    pub async fn update_comment(
        &self,
        issue_id_or_key: &str,
        comment_id: &str,
        body: &str,
    ) -> Result<JiraComment, ApiError> {
        let comment = self
            .generated
            .update_comment()
            .issue_id_or_key(issue_id_or_key)
            .id(comment_id)
            .body(generated_types::CommentCreateRequest {
                body: adf_body(body),
            })
            .send()
            .await
            .or_api_error()
            .await?;

        Ok(comment.into_inner().into())
    }

    pub async fn list_comments(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
    ) -> Result<JiraCommentPage, ApiError> {
        self.list_comments_from(issue_id_or_key, max_results, 0)
            .await
    }

    pub async fn list_comments_from(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
        start_at: u64,
    ) -> Result<JiraCommentPage, ApiError> {
        let max_results = self.raw_client.effective_item_limit(max_results);
        let mut collected: Vec<JiraComment> = Vec::new();
        let initial_start_at = start_at;
        let mut start_at: u64 = start_at;
        let mut last_total: Option<u32> = None;

        while self.raw_client.take_page() {
            let remaining = (max_results as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(JIRA_LIST_PAGE_CAP as u64) as u32;

            let page: JiraCommentPage = self
                .generated
                .get_comments()
                .issue_id_or_key(issue_id_or_key)
                .start_at(limit_i32(start_at.min(i32::MAX as u64) as u32))
                .max_results(limit_i32(page_size))
                .send()
                .await
                .or_api_error()
                .await?
                .into_inner()
                .into();

            let received = page.comments.len() as u64;
            last_total = page.total;
            collected.extend(page.comments);

            match next_offset(
                collected.len() as u64,
                max_results as u64,
                received,
                None,
                last_total.map(u64::from),
                start_at,
            ) {
                Some(next) => start_at = next,
                None => break,
            }
        }

        if collected.len() > max_results as usize {
            collected.truncate(max_results as usize);
        }

        Ok(JiraCommentPage {
            start_at: initial_start_at.min(u32::MAX as u64) as u32,
            max_results,
            total: last_total,
            comments: collected,
        })
    }

    pub async fn delete_comment(
        &self,
        issue_id_or_key: &str,
        comment_id: &str,
    ) -> Result<(), ApiError> {
        self.generated
            .delete_comment()
            .issue_id_or_key(issue_id_or_key)
            .id(comment_id)
            .send()
            .await
            .map(|_| ())
            .or_api_error()
            .await
    }
}
