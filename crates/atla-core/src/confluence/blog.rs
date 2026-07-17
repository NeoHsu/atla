use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_blog_posts(
        &self,
        search: &ConfluenceBlogPostSearch,
    ) -> Result<ConfluenceBlogPostPage, ApiError> {
        let limit = self.raw_client.effective_item_limit(search.limit);
        let mut collected: Vec<ConfluenceBlogPost> = Vec::new();
        let mut cursor: Option<String> = search.cursor.clone();
        let mut next_link: Option<String> = None;

        while self.raw_client.take_page() {
            let remaining = (limit as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(CONFLUENCE_LIST_PAGE_CAP as u64) as u32;

            let mut request = self
                .generated
                .get_blog_posts()
                .limit(limit_non_zero(page_size)?);
            if let Some(space_id) = optional_i64_vec(search.space_id.as_deref())? {
                request = request.space_id(space_id);
            }
            if let Some(title) = &search.title {
                request = request.title(title.clone());
            }
            if let Some(cursor) = &cursor {
                request = request.cursor(cursor.clone());
            }
            let page = request.send().await.or_api_error().await?.into_inner();

            let received = page.results.len();
            collected.extend(page.results.into_iter().map(ConfluenceBlogPost::from));
            next_link = page.links.as_ref().and_then(|l| l.next.clone());

            if received == 0 {
                break;
            }
            cursor = match next_link.as_deref().and_then(cursor_from_next_link) {
                Some(c) => Some(c),
                None => break,
            };
        }

        if collected.len() > limit as usize {
            collected.truncate(limit as usize);
        }

        let next_cursor = next_link.as_deref().and_then(cursor_from_next_link);
        Ok(ConfluenceBlogPostPage {
            results: collected,
            is_last: Some(next_cursor.is_none()),
            next_cursor,
        })
    }

    pub async fn get_blog_post(&self, id: &str) -> Result<ConfluenceBlogPost, ApiError> {
        self.get_blog_post_with_body_format(id, Some(ConfluenceBodyRepresentation::Storage))
            .await
    }

    pub async fn get_blog_post_with_body_format(
        &self,
        id: &str,
        body_format: Option<ConfluenceBodyRepresentation>,
    ) -> Result<ConfluenceBlogPost, ApiError> {
        let mut request = self
            .generated
            .get_blog_post_by_id()
            .id(parse_i64_id(id)?)
            .include_version(true);
        if let Some(body_format) = body_format {
            request = request.body_format(body_format.as_primary_body_single());
        }
        let post = request.send().await.or_api_error().await?.into_inner();

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
            .or_api_error()
            .await?;
        Ok(())
    }
}
