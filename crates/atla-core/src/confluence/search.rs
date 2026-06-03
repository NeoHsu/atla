use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn search(
        &self,
        search: &ConfluenceSearch,
    ) -> Result<ConfluenceSearchPage, ApiError> {
        let limit = search.limit.max(1);
        let mut collected: Vec<ConfluenceSearchResult> = Vec::new();
        let mut start: i32 = 0;
        let mut last_total: Option<u64> = None;

        loop {
            let remaining = (limit as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(CONFLUENCE_LIST_PAGE_CAP as u64) as u32;

            let raw = self
                .generated_v1
                .search_by_cql()
                .cql(&search.cql)
                .limit(limit_i32(page_size))
                .start(start)
                .send()
                .await
                .map_err(generated_v1_error)?
                .into_inner();

            let received = raw.results.len() as i32;
            let total = u64::try_from(raw.total_size).ok();
            last_total = total;
            let page: ConfluenceSearchPage = raw.into();
            collected.extend(page.results);

            if received == 0 {
                break;
            }
            start = match start.checked_add(received) {
                Some(next) if total.is_some_and(|t| next as u64 >= t) => break,
                Some(next) => next,
                None => break,
            };
            if total.is_some_and(|t| collected.len() as u64 >= t) {
                break;
            }
        }

        let exhausted = last_total.is_some_and(|total| collected.len() as u64 >= total);
        if collected.len() > limit as usize {
            collected.truncate(limit as usize);
        }

        Ok(ConfluenceSearchPage {
            results: collected,
            size: Some(0),
            total_size: last_total,
            is_last: Some(exhausted),
        })
    }
}
