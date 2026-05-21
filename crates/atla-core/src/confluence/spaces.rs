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

    pub async fn get_space_by_id(&self, id: &str) -> Result<Option<ConfluenceSpace>, ApiError> {
        let page = self
            .generated
            .get_spaces()
            .ids(vec![parse_i64_id(id)?])
            .limit(limit_non_zero(1)?)
            .send()
            .await
            .map_err(generated_error)?
            .into_inner();

        Ok(page.results.into_iter().next().map(ConfluenceSpace::from))
    }

    pub async fn get_space(&self, key_or_id: &str) -> Result<Option<ConfluenceSpace>, ApiError> {
        if let Some(space) = self.get_space_by_key(key_or_id).await? {
            return Ok(Some(space));
        }

        if key_or_id.parse::<i64>().is_ok() {
            return self.get_space_by_id(key_or_id).await;
        }

        Ok(None)
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

        match self
            .generated
            .create_space()
            .body(space.to_generated())
            .send()
            .await
        {
            Ok(rv) => Ok(ConfluenceSpace::from(rv.into_inner())),
            Err(e) => Err(generated_error_with_body(e).await),
        }
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

        let _space = match self
            .generated_v1
            .update_space()
            .space_key(&space.key)
            .body(space.to_v1_update_request())
            .send()
            .await
        {
            Ok(rv) => rv,
            Err(e) => return Err(generated_v1_error_with_body(e).await),
        };

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
