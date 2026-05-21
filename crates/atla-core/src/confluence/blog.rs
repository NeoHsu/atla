use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_blog_posts(
        &self,
        search: &ConfluenceBlogPostSearch,
    ) -> Result<ConfluenceBlogPostPage, ApiError> {
        let mut request = self
            .generated
            .get_blog_posts()
            .limit(limit_non_zero(search.limit)?);
        if let Some(space_id) = optional_i64_vec(search.space_id.as_deref())? {
            request = request.space_id(space_id);
        }
        if let Some(title) = &search.title {
            request = request.title(title.clone());
        }
        let page = request.send().await.map_err(generated_error)?.into_inner();

        Ok(ConfluenceBlogPostPage {
            results: page
                .results
                .into_iter()
                .map(ConfluenceBlogPost::from)
                .collect(),
        })
    }

    pub async fn get_blog_post(&self, id: &str) -> Result<ConfluenceBlogPost, ApiError> {
        let post = self
            .generated
            .get_blog_post_by_id()
            .id(parse_i64_id(id)?)
            .body_format(atla_confluence_api::types::PrimaryBodyRepresentationSingle::Storage)
            .include_version(true)
            .send()
            .await
            .map_err(generated_error)?
            .into_inner();

        Ok(post.into())
    }

    pub async fn create_blog_post(
        &self,
        post: &ConfluenceBlogPostCreate,
    ) -> Result<ConfluenceBlogPost, ApiError> {
        let mut request = self.generated.create_blog_post().body(post.to_generated());
        if let Some(private) = post.private {
            request = request.private(private);
        }
        let post = match request.send().await {
            Ok(rv) => rv.into_inner(),
            Err(e) => return Err(generated_error_with_body(e).await),
        };

        Ok(post.into())
    }

    pub async fn update_blog_post(
        &self,
        post: &ConfluenceBlogPostUpdate,
    ) -> Result<ConfluenceBlogPost, ApiError> {
        let post = match self
            .generated
            .update_blog_post()
            .id(parse_i64_id(&post.id)?)
            .body(post.to_generated())
            .send()
            .await
        {
            Ok(rv) => rv.into_inner(),
            Err(e) => return Err(generated_error_with_body(e).await),
        };

        Ok(post.into())
    }

    pub async fn delete_blog_post(
        &self,
        id: &str,
        purge: bool,
        draft: bool,
    ) -> Result<(), ApiError> {
        self.generated
            .delete_blog_post()
            .id(parse_i64_id(id)?)
            .purge(purge)
            .draft(draft)
            .send()
            .await
            .map_err(generated_error)?;
        Ok(())
    }
}
