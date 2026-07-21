use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_page_labels(
        &self,
        search: &ConfluenceLabelSearch,
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let content_id = parse_i64_id(&search.content_id)?;
        let limit = self.raw_client.effective_item_limit(search.limit);
        let mut collected: Vec<ConfluenceLabel> = Vec::new();
        let mut cursor: Option<String> = search.cursor.clone();
        let mut next_link: Option<String> = None;

        while self.raw_client.take_page() {
            let remaining = (limit as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(CONFLUENCE_LIST_PAGE_CAP as u64) as u32;

            let page_limit = limit_non_zero(page_size)?;
            let raw = generated_request(reqwest::Method::GET, || {
                let mut request = self
                    .generated
                    .get_page_labels()
                    .id(content_id)
                    .limit(page_limit);
                if let Some(prefix) = &search.prefix {
                    request = request.prefix(prefix.clone());
                }
                if let Some(cursor) = &cursor {
                    request = request.cursor(cursor.clone());
                }
                request.send()
            })
            .await?
            .into_inner();

            let received = raw.results.len();
            collected.extend(raw.results.into_iter().map(ConfluenceLabel::from));
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
        Ok(ConfluenceLabelPage {
            results: collected,
            is_last: Some(next_cursor.is_none()),
            next_cursor,
        })
    }

    pub async fn list_blog_labels(
        &self,
        search: &ConfluenceLabelSearch,
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let content_id = parse_i64_id(&search.content_id)?;
        let limit = self.raw_client.effective_item_limit(search.limit);
        let mut collected: Vec<ConfluenceLabel> = Vec::new();
        let mut cursor: Option<String> = search.cursor.clone();
        let mut next_link: Option<String> = None;

        while self.raw_client.take_page() {
            let remaining = (limit as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(CONFLUENCE_LIST_PAGE_CAP as u64) as u32;

            let page_limit = limit_non_zero(page_size)?;
            let raw = generated_request(reqwest::Method::GET, || {
                let mut request = self
                    .generated
                    .get_blog_post_labels()
                    .id(content_id)
                    .limit(page_limit);
                if let Some(prefix) = &search.prefix {
                    request = request.prefix(prefix.clone());
                }
                if let Some(cursor) = &cursor {
                    request = request.cursor(cursor.clone());
                }
                request.send()
            })
            .await?
            .into_inner();

            let received = raw.results.len();
            collected.extend(raw.results.into_iter().map(ConfluenceLabel::from));
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
        Ok(ConfluenceLabelPage {
            results: collected,
            is_last: Some(next_cursor.is_none()),
            next_cursor,
        })
    }

    pub async fn add_page_labels(
        &self,
        content_id: &str,
        labels: &[String],
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let labels = generated_request(reqwest::Method::POST, || {
            let body = atla_confluence_v1_api::types::LabelCreateArray(
                labels
                    .iter()
                    .map(|label| atla_confluence_v1_api::types::LabelCreate {
                        prefix: "global".to_owned(),
                        name: label.clone(),
                    })
                    .collect(),
            );
            self.generated_v1
                .add_labels_to_content()
                .id(content_id)
                .body(body)
                .send()
        })
        .await?
        .into_inner();

        Ok(labels.into())
    }

    pub async fn remove_page_label(&self, content_id: &str, label: &str) -> Result<(), ApiError> {
        generated_request(reqwest::Method::DELETE, || {
            self.generated_v1
                .remove_label_from_content_using_query_parameter()
                .id(content_id)
                .name(label)
                .send()
        })
        .await?;
        Ok(())
    }

    pub async fn add_blog_labels(
        &self,
        content_id: &str,
        labels: &[String],
    ) -> Result<ConfluenceLabelPage, ApiError> {
        self.add_page_labels(content_id, labels).await
    }

    pub async fn remove_blog_label(&self, content_id: &str, label: &str) -> Result<(), ApiError> {
        self.remove_page_label(content_id, label).await
    }
}
