use atla_jira_api::types as generated_types;

use super::JiraClient;
use super::models::{JiraComment, JiraCommentPage};
use super::util::{JIRA_LIST_PAGE_CAP, adf_body, generated_error, limit_i32, next_offset};
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
        let max_results = max_results.max(1);
        let mut collected: Vec<JiraComment> = Vec::new();
        let mut start_at: u64 = 0;
        let mut last_total: Option<u32> = None;

        loop {
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
                .map_err(generated_error)?
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
            start_at: 0,
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
            .map_err(generated_error)
    }
}
