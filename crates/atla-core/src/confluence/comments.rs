use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_page_comments(
        &self,
        search: &ConfluenceCommentSearch,
    ) -> Result<ConfluenceCommentPage, ApiError> {
        let page = self
            .generated
            .get_page_footer_comments()
            .id(parse_i64_id(&search.content_id)?)
            .body_format(atla_confluence_api::types::PrimaryBodyRepresentation::Storage)
            .limit(limit_non_zero(search.limit)?)
            .send()
            .await
            .map_err(generated_error)?
            .into_inner();

        Ok(page.into())
    }

    pub async fn add_page_comment(
        &self,
        comment: &ConfluenceCommentCreate,
    ) -> Result<ConfluenceComment, ApiError> {
        let created = self
            .generated
            .create_footer_comment()
            .body(comment.to_generated_page_footer())
            .send()
            .await
            .map_err(generated_error)?
            .into_inner();

        Ok(created.into())
    }

    pub async fn delete_page_comment(&self, comment_id: &str) -> Result<(), ApiError> {
        self.generated
            .delete_footer_comment()
            .comment_id(parse_i64_id(comment_id)?)
            .send()
            .await
            .map_err(generated_error)?;
        Ok(())
    }

    pub async fn list_blog_comments(
        &self,
        search: &ConfluenceCommentSearch,
    ) -> Result<ConfluenceCommentPage, ApiError> {
        let page = self
            .generated
            .get_blog_post_footer_comments()
            .id(parse_i64_id(&search.content_id)?)
            .body_format(atla_confluence_api::types::PrimaryBodyRepresentation::Storage)
            .limit(limit_non_zero(search.limit)?)
            .send()
            .await
            .map_err(generated_error)?
            .into_inner();

        Ok(page.into())
    }

    pub async fn add_blog_comment(
        &self,
        comment: &ConfluenceCommentCreate,
    ) -> Result<ConfluenceComment, ApiError> {
        let created = self
            .generated
            .create_footer_comment()
            .body(comment.to_generated_blog_footer())
            .send()
            .await
            .map_err(generated_error)?
            .into_inner();

        Ok(created.into())
    }
}
