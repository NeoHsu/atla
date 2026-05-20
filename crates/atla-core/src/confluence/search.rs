use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn search(
        &self,
        search: &ConfluenceSearch,
    ) -> Result<ConfluenceSearchPage, ApiError> {
        let page = self
            .generated_v1
            .search_by_cql()
            .cql(&search.cql)
            .limit(limit_i32(search.limit))
            .send()
            .await
            .map_err(generated_v1_error)?
            .into_inner();

        Ok(page.into())
    }
}
