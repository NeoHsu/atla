use std::path::{Path, PathBuf};

use atla_confluence_v1_api::types as generated_v1_types;

use crate::client::{ApiError, read_json};

use super::ConfluenceClient;
use super::models::*;
use super::util::*;

impl ConfluenceClient {
    pub async fn upload_page_attachment(
        &self,
        upload: &ConfluenceAttachmentUpload,
    ) -> Result<ConfluenceAttachmentPage, ApiError> {
        let filename = upload
            .file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("attachment")
            .to_owned();
        let content = std::fs::read(&upload.file).map_err(|error| {
            ApiError::Io(format!(
                "failed to read file `{}`: {error}",
                upload.file.display()
            ))
        })?;
        let part = reqwest::multipart::Part::bytes(content)
            .file_name(filename)
            .mime_str("application/octet-stream")
            .map_err(|error| ApiError::Decode(format!("invalid MIME type: {error}")))?;
        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("minorEdit", upload.minor_edit.to_string());
        if let Some(comment) = &upload.comment {
            form = form.text("comment", comment.clone());
        }

        let page: generated_v1_types::ContentArray = read_json(
            self.raw_client
                .put(&format!(
                    "/wiki/rest/api/content/{}/child/attachment",
                    upload.page_id
                ))
                .header("X-Atlassian-Token", "nocheck")
                .query(&[("status", "current")])
                .multipart(form),
        )
        .await?;

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
        let mut request = self
            .generated
            .get_page_attachments()
            .id(parse_i64_id(&search.page_id)?)
            .limit(limit_non_zero(search.limit)?);
        if let Some(filename) = &search.filename {
            request = request.filename(filename.clone());
        }
        let page = request.send().await.map_err(generated_error)?.into_inner();

        Ok(ConfluenceAttachmentPage {
            results: page
                .results
                .into_iter()
                .map(ConfluenceAttachment::from)
                .collect(),
        })
    }

    pub async fn get_attachment(&self, id: &str) -> Result<ConfluenceAttachment, ApiError> {
        self.generated
            .get_attachment_by_id()
            .id(id.to_owned())
            .include_version(true)
            .send()
            .await
            .map(|rv| ConfluenceAttachment::from(rv.into_inner()))
            .map_err(generated_error)
    }

    pub async fn delete_attachment(&self, id: &str, purge: bool) -> Result<(), ApiError> {
        self.generated
            .delete_attachment()
            .id(parse_i64_id(id)?)
            .purge(purge)
            .send()
            .await
            .map_err(generated_error)?;
        Ok(())
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
