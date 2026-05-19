use atla_confluence_api::apis as generated_apis;
use atla_confluence_v1_api::apis as generated_v1_apis;

use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_spaces(
        &self,
        search: &ConfluenceSpaceSearch,
    ) -> Result<ConfluenceSpacePage, ApiError> {
        let keys = search.key.as_ref().map(|key| vec![key.clone()]);
        let page = generated_apis::space_api::get_spaces(
            &self.generated,
            None,
            keys,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluenceSpacePage {
            results: page
                .results
                .unwrap_or_default()
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

        generated_apis::space_api::create_space(&self.generated, space.to_generated())
            .await
            .map(ConfluenceSpace::from)
            .map_err(generated_error)
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

        let _space = generated_v1_apis::space_api::update_space(
            &self.generated_v1,
            &space.key,
            space.to_v1_update_request(),
        )
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
        let _task = generated_v1_apis::space_api::delete_space(&self.generated_v1, key)
            .await
            .map_err(generated_v1_error)?;
        Ok(())
    }
}
