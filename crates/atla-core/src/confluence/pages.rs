use atla_confluence_api::{apis as generated_apis, models as generated_models};

use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_pages(
        &self,
        search: &ConfluencePageSearch,
    ) -> Result<ConfluencePagePage, ApiError> {
        let space_id = optional_i64_vec(search.space_id.as_deref())?;
        let page = generated_apis::page_api::get_pages(
            &self.generated,
            None,
            space_id,
            None,
            None,
            search.title.as_deref(),
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluencePagePage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluencePage::from)
                .collect(),
        })
    }

    pub async fn list_page_children(
        &self,
        search: &ConfluenceContentTreeSearch,
    ) -> Result<ConfluenceContentTreePage, ApiError> {
        let id = parse_i64_id(&search.page_id)?;
        if let Some(depth) = search.depth {
            let page = generated_apis::descendants_api::get_page_descendants(
                &self.generated,
                id,
                Some(limit_i32(search.limit)),
                Some(limit_i32(depth)),
                None,
            )
            .await
            .map_err(generated_error)?;

            return Ok(ConfluenceContentTreePage {
                results: page
                    .results
                    .unwrap_or_default()
                    .into_iter()
                    .map(ConfluenceContentNode::from)
                    .collect(),
            });
        }

        let page = generated_apis::children_api::get_page_direct_children(
            &self.generated,
            id,
            None,
            Some(limit_i32(search.limit)),
            None,
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluenceContentTreePage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceContentNode::from)
                .collect(),
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
        let page = generated_apis::page_api::get_page_by_id(
            &self.generated,
            parse_i64_id(id)?,
            body_format.map(ConfluenceBodyRepresentation::as_primary_body_single),
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
            None,
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn create_page(
        &self,
        page: &ConfluencePageCreate,
    ) -> Result<ConfluencePage, ApiError> {
        let page = generated_apis::page_api::create_page(
            &self.generated,
            page.to_generated(),
            None,
            page.private,
            page.root_level,
        )
        .await
        .map_err(generated_error)?;

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
        let page = generated_apis::page_api::update_page_title(
            &self.generated,
            parse_i64_id(id)?,
            generated_models::UpdatePageTitleRequest::new(
                status.into_update_page_title_status(),
                title.to_owned(),
            ),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn update_page(
        &self,
        page: &ConfluencePageUpdate,
    ) -> Result<ConfluencePage, ApiError> {
        let updated = generated_apis::page_api::update_page(
            &self.generated,
            parse_i64_id(&page.id)?,
            page.to_generated(),
        )
        .await
        .map_err(generated_error)?;

        Ok(updated.into())
    }

    pub async fn delete_page(&self, id: &str, purge: bool, draft: bool) -> Result<(), ApiError> {
        generated_apis::page_api::delete_page(
            &self.generated,
            parse_i64_id(id)?,
            Some(purge),
            Some(draft),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn move_page(&self, id: &str, parent_id: &str) -> Result<ConfluencePage, ApiError> {
        let existing = generated_apis::page_api::get_page_by_id(
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
            None,
        )
        .await
        .map_err(generated_error)?;

        let body = existing
            .body
            .as_ref()
            .and_then(|body| body.storage.as_ref())
            .and_then(|body| body.value.clone())
            .ok_or_else(|| {
                ApiError::Decode(format!("page `{id}` did not include a storage body"))
            })?;
        let title = existing
            .title
            .clone()
            .ok_or_else(|| ApiError::Decode(format!("page `{id}` did not include a title")))?;
        let version = existing
            .version
            .as_ref()
            .and_then(|version| version.number)
            .ok_or_else(|| ApiError::Decode(format!("page `{id}` did not include a version")))?;
        let status = match existing.status {
            Some(generated_models::ContentStatus::Draft) => ConfluenceContentStatus::Draft,
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
            version: version as u64 + 1,
            message: Some(format!("Move under {parent_id}")),
        })
        .await
    }
}
