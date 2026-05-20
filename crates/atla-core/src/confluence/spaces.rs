use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_spaces(
        &self,
        search: &ConfluenceSpaceSearch,
    ) -> Result<ConfluenceSpacePage, ApiError> {
        let mut request = self
            .generated
            .get_spaces()
            .limit(limit_non_zero(search.limit)?);
        if let Some(key) = &search.key {
            request = request.keys(vec![key.clone()]);
        }
        let page = request.send().await.map_err(generated_error)?.into_inner();

        Ok(ConfluenceSpacePage {
            results: page
                .results
                .into_iter()
                .map(ConfluenceSpace::from)
                .collect(),
        })
    }

    pub async fn get_space_by_key(&self, key: &str) -> Result<Option<ConfluenceSpace>, ApiError> {
        let page = self
            .list_spaces(&ConfluenceSpaceSearch {
                key: Some(key.to_owned()),
                limit: 1,
            })
            .await?;

        Ok(page.results.into_iter().next())
    }

    pub async fn create_space(
        &self,
        space: &ConfluenceSpaceCreate,
    ) -> Result<ConfluenceSpace, ApiError> {
        if space.key.is_none() && space.alias.is_none() {
            return Err(ApiError::Decode(
                "Confluence space create requires a key or alias".to_owned(),
            ));
        }

        self.generated
            .create_space()
            .body(space.to_generated())
            .send()
            .await
            .map_err(generated_error)
            .map(|rv| ConfluenceSpace::from(rv.into_inner()))
    }

    pub async fn update_space(
        &self,
        space: &ConfluenceSpaceUpdate,
    ) -> Result<ConfluenceSpace, ApiError> {
        if space.name.is_none() && space.description.is_none() {
            return Err(ApiError::Decode(
                "Confluence space update requires at least one field".to_owned(),
            ));
        }

        let _space = self
            .generated_v1
            .update_space()
            .space_key(&space.key)
            .body(space.to_v1_update_request())
            .send()
            .await
            .map_err(generated_v1_error)?;

        self.get_space_by_key(&space.key).await?.ok_or_else(|| {
            ApiError::Decode(format!(
                "Confluence space `{}` was updated but could not be loaded",
                space.key
            ))
        })
    }

    pub async fn delete_space(&self, key: &str) -> Result<(), ApiError> {
        let _task = self
            .generated_v1
            .delete_space()
            .space_key(key)
            .send()
            .await
            .map_err(generated_v1_error)?;
        Ok(())
    }
}
