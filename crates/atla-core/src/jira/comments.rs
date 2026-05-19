use atla_jira_api::apis as generated_apis;

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
        let comment = generated_apis::issue_comments_api::add_comment(
            &self.generated,
            issue_id_or_key,
            atla_jira_api::models::CommentCreateRequest::new(adf_body(body)),
        )
        .await
        .map_err(generated_error)?;

        Ok(comment.into())
    }

    pub async fn update_comment(
        &self,
        issue_id_or_key: &str,
        comment_id: &str,
        body: &str,
    ) -> Result<JiraComment, ApiError> {
        let comment = generated_apis::issue_comments_api::update_comment(
            &self.generated,
            issue_id_or_key,
            comment_id,
            atla_jira_api::models::CommentCreateRequest::new(adf_body(body)),
        )
        .await
        .map_err(generated_error)?;

        Ok(comment.into())
    }

    pub async fn list_comments(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
    ) -> Result<JiraCommentPage, ApiError> {
        let page = generated_apis::issue_comments_api::get_comments(
            &self.generated,
            issue_id_or_key,
            Some(0),
            Some(limit_i32(max_results)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn delete_comment(
        &self,
        issue_id_or_key: &str,
        comment_id: &str,
    ) -> Result<(), ApiError> {
        generated_apis::issue_comments_api::delete_comment(
            &self.generated,
            issue_id_or_key,
            comment_id,
        )
        .await
        .map_err(generated_error)
    }
}
