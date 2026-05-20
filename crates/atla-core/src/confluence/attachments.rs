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
            .id(id.strip_prefix("att").unwrap_or(id).to_owned())
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
        let attachment_id = parse_i64_id(id)?;
        // Build the v1 REST API fallback: requires both the content (page/blog) ID and
        // the attachment ID. This endpoint supports API-token basic auth, unlike the
        // download servlet (/wiki/download/attachments/...) which only accepts session auth.
        let fallback_download_link = attachment
            .page_id
            .as_deref()
            .or(attachment.blog_post_id.as_deref())
            .and_then(|content_id| parse_i64_id(content_id).ok())
            .map(|content_id| {
                format!(
                    "/wiki/rest/api/content/{content_id}/child/attachment/{attachment_id}/download"
                )
            })
            .unwrap_or_else(|| format!("/wiki/rest/api/content/{attachment_id}/download"));
        let mut download_links = attachment
            .download_link
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        if !download_links
            .iter()
            .any(|link| link == &fallback_download_link)
        {
            download_links.push(fallback_download_link);
        }
        let mut last_error = None;
        let mut bytes = None;
        for (index, download_link) in download_links.iter().enumerate() {
            // Confluence download paths like `/download/attachments/...` are relative to
            // the wiki root, so they need a `/wiki` prefix when used with the base URL.
            let normalized = if download_link.starts_with("/download/") {
                format!("/wiki{download_link}")
            } else {
                download_link.clone()
            };
            let response = self.raw_client.get_binary(&normalized).send().await?;
            let status = response.status();
            if status.is_success() {
                bytes = Some(response.bytes().await.map_err(ApiError::Request)?);
                break;
            }

            let body = response.text().await.unwrap_or_default();
            let error = ApiError::Http {
                status,
                body: crate::client::extract_api_error_body(&body),
            };
            let has_fallback = index + 1 < download_links.len();
            // The Confluence download servlet (/wiki/download/attachments/...) rejects
            // API-token basic auth with 401. Try the REST API fallback on auth failures too.
            if (status == reqwest::StatusCode::NOT_FOUND
                || status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN)
                && has_fallback
            {
                last_error = Some(error);
                continue;
            }
            return Err(error);
        }
        let bytes = bytes.ok_or_else(|| {
            last_error.unwrap_or_else(|| {
                ApiError::Decode(format!(
                    "attachment `{id}` did not include a usable download link"
                ))
            })
        })?;
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
