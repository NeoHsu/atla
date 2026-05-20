use atla_jira_api::types as generated_types;

use super::JiraClient;
use super::models::{JiraComment, JiraCommentPage};
use super::util::{adf_body, generated_error, limit_i32};
use crate::client::ApiError;

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
            .map_err(generated_error)?;

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
            .map_err(generated_error)?;

        Ok(comment.into_inner().into())
    }

    pub async fn list_comments(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
    ) -> Result<JiraCommentPage, ApiError> {
        let page = self
            .generated
            .get_comments()
            .issue_id_or_key(issue_id_or_key)
            .start_at(0)
            .max_results(limit_i32(max_results))
            .send()
            .await
            .map_err(generated_error)?;

        Ok(page.into_inner().into())
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
            .map_err(generated_error)
    }
}
