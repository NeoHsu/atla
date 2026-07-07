use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_pages(
        &self,
        search: &ConfluencePageSearch,
    ) -> Result<ConfluencePagePage, ApiError> {
        let limit = search.limit.max(1);
        let mut collected: Vec<ConfluencePage> = Vec::new();
        let mut cursor: Option<String> = search.cursor.clone();
        let mut next_link: Option<String> = None;

        loop {
            let remaining = (limit as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(CONFLUENCE_LIST_PAGE_CAP as u64) as u32;

            let mut request = self.generated.get_pages().limit(limit_non_zero(page_size)?);
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
            collected.extend(page.results.into_iter().map(ConfluencePage::from));
            next_link = page.links.as_ref().and_then(|links| links.next.clone());

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
        Ok(ConfluencePagePage {
            results: collected,
            is_last: Some(next_cursor.is_none()),
            next_cursor,
        })
    }

    pub async fn list_page_children(
        &self,
        search: &ConfluenceContentTreeSearch,
    ) -> Result<ConfluenceContentTreePage, ApiError> {
        let id = parse_i64_id(&search.page_id)?;
        let limit = search.limit.max(1);
        let mut collected: Vec<ConfluenceContentNode> = Vec::new();
        let mut cursor: Option<String> = search.cursor.clone();
        let mut next_link: Option<String> = None;

        loop {
            let remaining = (limit as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(CONFLUENCE_LIST_PAGE_CAP as u64) as u32;

            let (nodes, links) = if let Some(depth) = search.depth {
                let page = {
                    let mut req = self
                        .generated
                        .get_page_descendants()
                        .id(id)
                        .limit(limit_non_zero(page_size)?)
                        .depth(limit_non_zero(depth)?);
                    if let Some(cursor) = &cursor {
                        req = req.cursor(cursor.clone());
                    }
                    req.send().await.or_api_error().await?.into_inner()
                };
                let nodes: Vec<ConfluenceContentNode> = page
                    .results
                    .into_iter()
                    .map(ConfluenceContentNode::from)
                    .collect();
                (nodes, page.links)
            } else {
                let page = {
                    let mut req = self
                        .generated
                        .get_page_direct_children()
                        .id(id)
                        .limit(limit_non_zero(page_size)?);
                    if let Some(cursor) = &cursor {
                        req = req.cursor(cursor.clone());
                    }
                    req.send().await.or_api_error().await?.into_inner()
                };
                let nodes: Vec<ConfluenceContentNode> = page
                    .results
                    .into_iter()
                    .map(ConfluenceContentNode::from)
                    .collect();
                (nodes, page.links)
            };

            let received = nodes.len();
            collected.extend(nodes);
            next_link = links.as_ref().and_then(|l| l.next.clone());

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
        Ok(ConfluenceContentTreePage {
            results: collected,
            is_last: Some(next_cursor.is_none()),
            next_cursor,
        })
    }

    pub async fn get_page(&self, id: &str) -> Result<ConfluencePage, ApiError> {
        self.get_page_with_body_format(id, None).await
    }

    pub async fn get_page_with_body_format(
        &self,
        id: &str,
        body_format: Option<ConfluenceBodyRepresentation>,
    ) -> Result<ConfluencePage, ApiError> {
        let mut request = self
            .generated
            .get_page_by_id()
            .id(parse_i64_id(id)?)
            .include_version(true);
        if let Some(body_format) = body_format {
            request = request.body_format(body_format.as_primary_body_single());
        }
        let page = request.send().await.or_api_error().await?.into_inner();

        Ok(page.into())
    }

    pub async fn create_page(
        &self,
        page: &ConfluencePageCreate,
    ) -> Result<ConfluencePage, ApiError> {
        let mut request = self.generated.create_page().body(page.to_generated());
        if let Some(private) = page.private {
            request = request.private(private);
        }
        if let Some(root_level) = page.root_level {
            request = request.root_level(root_level);
        }
        let page = match request.send().await {
            Ok(rv) => rv.into_inner(),
            Err(e) => return Err(generated_error_with_body(e).await),
        };

        Ok(page.into())
    }

    pub async fn copy_page(&self, copy: &ConfluencePageCopy) -> Result<ConfluencePage, ApiError> {
        let source = self
            .get_page_with_body_format(&copy.source_id, Some(ConfluenceBodyRepresentation::Storage))
            .await?;
        let body = source.body.ok_or_else(|| {
            ApiError::Decode(format!(
                "Confluence page `{}` did not include storage body",
                copy.source_id
            ))
        })?;
        let space_id = copy.space_id.clone().or(source.space_id).ok_or_else(|| {
            ApiError::Decode(format!(
                "Confluence page `{}` did not include a space id; pass --space-id",
                copy.source_id
            ))
        })?;

        self.create_page(&ConfluencePageCreate {
            space_id,
            title: copy.title.clone(),
            parent_id: copy.parent_id.clone(),
            body: Some(body),
            representation: ConfluenceBodyRepresentation::Storage,
            status: ConfluenceContentStatus::Current,
            private: None,
            root_level: copy.root_level.then_some(true),
        })
        .await
    }

    pub async fn update_page_title(
        &self,
        id: &str,
        title: &str,
        status: ConfluenceContentStatus,
    ) -> Result<ConfluencePage, ApiError> {
        let page = self
            .generated
            .update_page_title()
            .id(parse_i64_id(id)?)
            .body(atla_confluence_api::types::UpdatePageTitleBody {
                status: status.into_update_page_title_status(),
                title: title.to_owned(),
            })
            .send()
            .await
            .or_api_error()
            .await?
            .into_inner();

        Ok(page.into())
    }

    pub async fn update_page(
        &self,
        page: &ConfluencePageUpdate,
    ) -> Result<ConfluencePage, ApiError> {
        let updated = match self
            .generated
            .update_page()
            .id(parse_i64_id(&page.id)?)
            .body(page.to_generated())
            .send()
            .await
        {
            Ok(rv) => rv.into_inner(),
            Err(e) => return Err(generated_error_with_body(e).await),
        };

        Ok(updated.into())
    }

    pub async fn delete_page(&self, id: &str, purge: bool, draft: bool) -> Result<(), ApiError> {
        self.generated
            .delete_page()
            .id(parse_i64_id(id)?)
            .purge(purge)
            .draft(draft)
            .send()
            .await
            .or_api_error()
            .await?;
        Ok(())
    }

    pub async fn move_page(&self, id: &str, parent_id: &str) -> Result<ConfluencePage, ApiError> {
        let existing = self
            .get_page_with_body_format(id, Some(ConfluenceBodyRepresentation::Storage))
            .await?;

        let body = existing.body.ok_or_else(|| {
            ApiError::Decode(format!("page `{id}` did not include a storage body"))
        })?;
        let title = existing
            .title
            .ok_or_else(|| ApiError::Decode(format!("page `{id}` did not include a title")))?;
        let version = existing
            .version
            .as_ref()
            .and_then(|version| version.number)
            .ok_or_else(|| ApiError::Decode(format!("page `{id}` did not include a version")))?;
        let status = match existing.status.as_deref() {
            Some("draft") => ConfluenceContentStatus::Draft,
            _ => ConfluenceContentStatus::Current,
        };

        self.update_page(&ConfluencePageUpdate {
            id: id.to_owned(),
            status,
            title,
            space_id: existing.space_id,
            parent_id: Some(parent_id.to_owned()),
            body,
            representation: ConfluenceBodyRepresentation::Storage,
            version: version + 1,
            message: Some("Move page".to_owned()),
        })
        .await
    }
}
