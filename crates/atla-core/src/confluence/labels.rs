use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn list_page_labels(
        &self,
        search: &ConfluenceLabelSearch,
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let mut request = self
            .generated
            .get_page_labels()
            .id(parse_i64_id(&search.content_id)?)
            .limit(limit_non_zero(search.limit)?);
        if let Some(prefix) = &search.prefix {
            request = request.prefix(prefix.clone());
        }
        let page = request.send().await.map_err(generated_error)?.into_inner();

        Ok(page.into())
    }

    pub async fn list_blog_labels(
        &self,
        search: &ConfluenceLabelSearch,
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let mut request = self
            .generated
            .get_blog_post_labels()
            .id(parse_i64_id(&search.content_id)?)
            .limit(limit_non_zero(search.limit)?);
        if let Some(prefix) = &search.prefix {
            request = request.prefix(prefix.clone());
        }
        let page = request.send().await.map_err(generated_error)?.into_inner();

        Ok(page.into())
    }

    pub async fn add_page_labels(
        &self,
        content_id: &str,
        labels: &[String],
    ) -> Result<ConfluenceLabelPage, ApiError> {
        let request = atla_confluence_v1_api::types::LabelCreateArray(
            labels
                .iter()
                .map(|label| atla_confluence_v1_api::types::LabelCreate {
                    prefix: "global".to_owned(),
                    name: label.clone(),
                })
                .collect(),
        );
        let labels = self
            .generated_v1
            .add_labels_to_content()
            .id(content_id)
            .body(request)
            .send()
            .await
            .map_err(generated_v1_error)?
            .into_inner();

        Ok(labels.into())
    }

    pub async fn remove_page_label(&self, content_id: &str, label: &str) -> Result<(), ApiError> {
        self.generated_v1
            .remove_label_from_content_using_query_parameter()
            .id(content_id)
            .name(label)
            .send()
            .await
            .map_err(generated_v1_error)?;
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
