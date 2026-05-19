use atla_confluence_api::apis as generated_apis;
use atla_confluence_v1_api::apis as generated_v1_apis;
use std::path::{Path, PathBuf};

use crate::client::ApiError;

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn upload_page_attachment(
        &self,
        upload: &ConfluenceAttachmentUpload,
    ) -> Result<ConfluenceAttachmentPage, ApiError> {
        let page = generated_v1_apis::content_attachments_api::create_or_update_attachments(
            &self.generated_v1,
            &upload.page_id,
            "nocheck",
            upload.file.clone(),
            upload.minor_edit,
            Some("current"),
            upload.comment.as_deref(),
        )
        .await
        .map_err(generated_v1_error)?;

        Ok(ConfluenceAttachmentPage {
            results: page
                .results
                .into_iter()
                .map(ConfluenceAttachment::from)
                .collect(),
        })
    }

    pub async fn list_page_attachments(
        &self,
        search: &ConfluenceAttachmentSearch,
    ) -> Result<ConfluenceAttachmentPage, ApiError> {
        let page = generated_apis::attachment_api::get_page_attachments(
            &self.generated,
            parse_i64_id(&search.page_id)?,
            None,
            None,
            None,
            None,
            search.filename.as_deref(),
            Some(limit_i32(search.limit)),
        )
        .await
        .map_err(generated_error)?;

        Ok(ConfluenceAttachmentPage {
            results: page
                .results
                .unwrap_or_default()
                .into_iter()
                .map(ConfluenceAttachment::from)
                .collect(),
        })
    }

    pub async fn get_attachment(&self, id: &str) -> Result<ConfluenceAttachment, ApiError> {
        generated_apis::attachment_api::get_attachment_by_id(
            &self.generated,
            id,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            None,
        )
        .await
        .map(ConfluenceAttachment::from)
        .map_err(generated_error)
    }

    pub async fn delete_attachment(&self, id: &str, purge: bool) -> Result<(), ApiError> {
        generated_apis::attachment_api::delete_attachment(
            &self.generated,
            parse_i64_id(id)?,
            Some(purge),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn download_attachment(
        &self,
        id: &str,
        output: Option<&Path>,
    ) -> Result<ConfluenceAttachmentDownload, ApiError> {
        let attachment = self.get_attachment(id).await?;
        let download_link = attachment.download_link.clone().ok_or_else(|| {
            ApiError::Decode(format!("attachment `{id}` did not include a downloadLink"))
        })?;
        let response = self.raw_client.get(&download_link).send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Http {
                status,
                body: crate::client::extract_api_error_body(&body),
            });
        }
        let bytes = response.bytes().await.map_err(ApiError::Request)?;
        let filename = output
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(attachment.title.as_deref().unwrap_or(id)));
        std::fs::write(&filename, &bytes).map_err(|error| {
            ApiError::Decode(format!("failed to write {}: {error}", filename.display()))
        })?;

        Ok(ConfluenceAttachmentDownload {
            attachment,
            path: filename,
            bytes: bytes.len() as u64,
        })
    }
}
