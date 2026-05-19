use atla_confluence_v1_api::apis as generated_v1_apis;

use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn search(
        &self,
        search: &ConfluenceSearch,
    ) -> Result<ConfluenceSearchPage, ApiError> {
        let page = generated_v1_apis::search_api::search_by_cql(
            &self.generated_v1,
            &search.cql,
            None,
            None,
            None,
            None,
            Some(limit_i32(search.limit)),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(generated_v1_error)?;

        Ok(page.into())
    }
}
