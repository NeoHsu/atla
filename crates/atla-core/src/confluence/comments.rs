use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_page_comments(
        &self,
        search: &ConfluenceCommentSearch,
    ) -> Result<ConfluenceCommentPage, ApiError> {
        let content_id = parse_i64_id(&search.content_id)?;
        let limit = search.limit.max(1);
        let mut collected: Vec<ConfluenceComment> = Vec::new();
        let mut cursor: Option<String> = search.cursor.clone();
        let mut next_link: Option<String> = None;

        loop {
            let remaining = (limit as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(CONFLUENCE_LIST_PAGE_CAP as u64) as u32;

            let mut req = self
                .generated
                .get_page_footer_comments()
                .id(content_id)
                .body_format(atla_confluence_api::types::PrimaryBodyRepresentation::Storage)
                .limit(limit_non_zero(page_size)?);
            if let Some(cursor) = &cursor {
                req = req.cursor(cursor.clone());
            }
            let raw = req.send().await.map_err(generated_error)?.into_inner();

            let received = raw.results.len();
            collected.extend(raw.results.into_iter().map(ConfluenceComment::from));
            next_link = raw.links.as_ref().and_then(|l| l.next.clone());

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
        Ok(ConfluenceCommentPage {
            results: collected,
            is_last: Some(next_cursor.is_none()),
            next_cursor,
        })
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
        let content_id = parse_i64_id(&search.content_id)?;
        let limit = search.limit.max(1);
        let mut collected: Vec<ConfluenceComment> = Vec::new();
        let mut cursor: Option<String> = search.cursor.clone();
        let mut next_link: Option<String> = None;

        loop {
            let remaining = (limit as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(CONFLUENCE_LIST_PAGE_CAP as u64) as u32;

            let mut req = self
                .generated
                .get_blog_post_footer_comments()
                .id(content_id)
                .body_format(atla_confluence_api::types::PrimaryBodyRepresentation::Storage)
                .limit(limit_non_zero(page_size)?);
            if let Some(cursor) = &cursor {
                req = req.cursor(cursor.clone());
            }
            let raw = req.send().await.map_err(generated_error)?.into_inner();

            let received = raw.results.len();
            collected.extend(raw.results.into_iter().map(ConfluenceComment::from));
            next_link = raw.links.as_ref().and_then(|l| l.next.clone());

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
        Ok(ConfluenceCommentPage {
            results: collected,
            is_last: Some(next_cursor.is_none()),
            next_cursor,
        })
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
