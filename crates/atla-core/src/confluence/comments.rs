use atla_confluence_api::{apis as generated_apis, models as generated_models};

use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_page_comments(
        &self,
        search: &ConfluenceCommentSearch,
    ) -> Result<ConfluenceCommentPage, ApiError> {
        let page = generated_apis::comment_api::get_page_footer_comments(
            &self.generated,
            parse_i64_id(&search.content_id)?,
            Some(generated_models::PrimaryBodyRepresentation::Storage),
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn add_page_comment(
        &self,
        comment: &ConfluenceCommentCreate,
    ) -> Result<ConfluenceComment, ApiError> {
        let created = generated_apis::comment_api::create_footer_comment(
            &self.generated,
            comment.to_generated_page_footer(),
        )
        .await
        .map_err(generated_error)?;

        Ok(created.into())
    }

    pub async fn delete_page_comment(&self, comment_id: &str) -> Result<(), ApiError> {
        generated_apis::comment_api::delete_footer_comment(
            &self.generated,
            parse_i64_id(comment_id)?,
        )
        .await
        .map_err(generated_error)
    }

    pub async fn list_blog_comments(
        &self,
        search: &ConfluenceCommentSearch,
    ) -> Result<ConfluenceCommentPage, ApiError> {
        let page = generated_apis::comment_api::get_blog_post_footer_comments(
            &self.generated,
            parse_i64_id(&search.content_id)?,
            Some(generated_models::PrimaryBodyRepresentation::Storage),
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn add_blog_comment(
        &self,
        comment: &ConfluenceCommentCreate,
    ) -> Result<ConfluenceComment, ApiError> {
        let created = generated_apis::comment_api::create_footer_comment(
            &self.generated,
            comment.to_generated_blog_footer(),
        )
        .await
        .map_err(generated_error)?;

        Ok(created.into())
    }
}
