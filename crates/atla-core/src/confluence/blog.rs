use atla_confluence_api::{apis as generated_apis, models as generated_models};

use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_blog_posts(
        &self,
        search: &ConfluenceBlogPostSearch,
    ) -> Result<ConfluenceBlogPostPage, ApiError> {
        let space_id = optional_i64_vec(search.space_id.as_deref())?;
        let page = generated_apis::blog_post_api::get_blog_posts(
            &self.generated,
            None,
            space_id,
            None,
            None,
            search.title.as_deref(),
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluenceBlogPostPage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceBlogPost::from)
                .collect(),
        })
    }

    pub async fn get_blog_post(&self, id: &str) -> Result<ConfluenceBlogPost, ApiError> {
        let post = generated_apis::blog_post_api::get_blog_post_by_id(
            &self.generated,
            parse_i64_id(id)?,
            Some(generated_models::PrimaryBodyRepresentationSingle::Storage),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        )
        .await
        .map_err(generated_error)?;

        Ok(post.into())
    }

    pub async fn create_blog_post(
        &self,
        post: &ConfluenceBlogPostCreate,
    ) -> Result<ConfluenceBlogPost, ApiError> {
        let post = generated_apis::blog_post_api::create_blog_post(
            &self.generated,
            post.to_generated(),
            post.private,
        )
        .await
        .map_err(generated_error)?;

        Ok(post.into())
    }

    pub async fn update_blog_post(
        &self,
        post: &ConfluenceBlogPostUpdate,
    ) -> Result<ConfluenceBlogPost, ApiError> {
        let post = generated_apis::blog_post_api::update_blog_post(
            &self.generated,
            parse_i64_id(&post.id)?,
            post.to_generated(),
        )
        .await
        .map_err(generated_error)?;

        Ok(post.into())
    }

    pub async fn delete_blog_post(
        &self,
        id: &str,
        purge: bool,
        draft: bool,
    ) -> Result<(), ApiError> {
        generated_apis::blog_post_api::delete_blog_post(
            &self.generated,
            parse_i64_id(id)?,
            Some(purge),
            Some(draft),
        )
        .await
        .map_err(generated_error)
    }
}
