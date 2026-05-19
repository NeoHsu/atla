use atla_confluence_api::apis as generated_apis;
use atla_confluence_v1_api::{apis as generated_v1_apis, models as generated_v1_models};

use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_page_labels(
        &self,
        search: &ConfluenceLabelSearch,
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let page = generated_apis::label_api::get_page_labels(
            &self.generated,
            parse_i64_id(&search.content_id)?,
            search.prefix.as_deref(),
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn list_blog_labels(
        &self,
        search: &ConfluenceLabelSearch,
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let page = generated_apis::label_api::get_blog_post_labels(
            &self.generated,
            parse_i64_id(&search.content_id)?,
            search.prefix.as_deref(),
            None,
            None,
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn add_page_labels(
        &self,
        content_id: &str,
        labels: &[String],
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let request = generated_v1_models::AddLabelsToContentRequest::LabelCreateArray(
            labels
                .iter()
                .map(|label| {
                    generated_v1_models::LabelCreate::new("global".to_owned(), label.clone())
                })
                .collect(),
        );
        let _labels = generated_v1_apis::content_labels_api::add_labels_to_content(
            &self.generated_v1,
            content_id,
            request,
        )
        .await
        .map_err(generated_v1_error)?;

        Ok(ConfluenceLabelPage {
            results: labels
                .iter()
                .map(|label| ConfluenceLabel {
                    id: None,
                    name: Some(label.clone()),
                    prefix: Some("global".to_owned()),
                })
                .collect(),
        })
    }

    pub async fn remove_page_label(&self, content_id: &str, label: &str) -> Result<(), ApiError> {
        generated_v1_apis::content_labels_api::remove_label_from_content_using_query_parameter(
            &self.generated_v1,
            content_id,
            label,
        )
        .await
        .map_err(generated_v1_error)
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
