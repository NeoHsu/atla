use atla_jira_api::apis as generated_apis;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::JiraClient;
use super::models::{JiraAttachment, JiraAttachmentDownload};
use super::util::generated_error;
use crate::client::{ApiError, read_json};

impl JiraClient {
    pub async fn get_attachment(&self, id: &str) -> Result<JiraAttachment, ApiError> {
        generated_apis::issue_attachments_api::get_attachment(&self.generated, id)
            .await
            .map(JiraAttachment::from)
            .map_err(generated_error)
    }

    pub async fn list_issue_attachments(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraAttachment>, ApiError> {
        let issue = self
            .get_issue(issue_id_or_key, Some(vec!["attachment".to_owned()]))
            .await?;
        issue
            .fields
            .get("attachment")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| ApiError::Decode(format!("failed to decode attachments: {error}")))
    }

    pub async fn upload_attachment(
        &self,
        issue_id_or_key: &str,
        file_path: &Path,
    ) -> Result<Vec<JiraAttachment>, ApiError> {
        let filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("attachment")
            .to_owned();
        let content = std::fs::read(file_path).map_err(|error| {
            ApiError::Io(format!(
                "failed to read file `{}`: {error}",
                file_path.display()
            ))
        })?;
        let part = reqwest::multipart::Part::bytes(content)
            .file_name(filename)
            .mime_str("application/octet-stream")
            .map_err(|error| ApiError::Decode(format!("invalid MIME type: {error}")))?;
        let form = reqwest::multipart::Form::new().part("file", part);

        read_json(
            self.raw_client
                .post_multipart(&format!("/rest/api/3/issue/{issue_id_or_key}/attachments"))
                .multipart(form),
        )
        .await
    }

    pub async fn delete_attachment(&self, attachment_id: &str) -> Result<(), ApiError> {
        generated_apis::issue_attachments_api::remove_attachment(&self.generated, attachment_id)
            .await
            .map_err(generated_error)
    }

    pub async fn download_attachment(
        &self,
        id: &str,
        output: Option<&Path>,
    ) -> Result<JiraAttachmentDownload, ApiError> {
        let attachment = self.get_attachment(id).await?;
        self.download_attachment_metadata(attachment, output).await
    }

    pub async fn download_issue_attachments(
        &self,
        issue_id_or_key: &str,
        output_dir: Option<&Path>,
    ) -> Result<Vec<JiraAttachmentDownload>, ApiError> {
        let attachments = self.list_issue_attachments(issue_id_or_key).await?;
        let mut downloads = Vec::new();
        let mut used_paths: HashSet<PathBuf> = HashSet::new();
        for attachment in attachments {
            let base_path = output_dir
                .map(|dir| dir.join(attachment_filename(&attachment)))
                .unwrap_or_else(|| PathBuf::from(attachment_filename(&attachment)));
            let final_path = deduplicate_path(&base_path, &used_paths);
            used_paths.insert(final_path.clone());
            downloads.push(
                self.download_attachment_metadata(attachment, Some(&final_path))
                    .await?,
            );
        }

        Ok(downloads)
    }

    async fn download_attachment_metadata(
        &self,
        attachment: JiraAttachment,
        output: Option<&Path>,
    ) -> Result<JiraAttachmentDownload, ApiError> {
        let id = attachment.id.as_deref().unwrap_or("attachment");
        let content = attachment.content.clone().ok_or_else(|| {
            ApiError::Decode(format!("attachment `{id}` did not include a content URL"))
        })?;
        let response = self.raw_client.get(&content).send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, body });
        }
        let bytes = response.bytes().await.map_err(ApiError::Request)?;
        let filename = attachment_filename(&attachment);
        let path = attachment_output_path(output, filename);
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|error| {
                ApiError::Decode(format!("failed to create {}: {error}", parent.display()))
            })?;
        }
        std::fs::write(&path, &bytes).map_err(|error| {
            ApiError::Decode(format!("failed to write {}: {error}", path.display()))
        })?;

        Ok(JiraAttachmentDownload {
            attachment,
            path,
            bytes: bytes.len() as u64,
        })
    }
}

fn attachment_output_path(output: Option<&Path>, filename: &str) -> PathBuf {
    match output {
        Some(output) if output.is_dir() => output.join(filename),
        Some(output) => output.to_path_buf(),
        None => PathBuf::from(filename),
    }
}

/// Appends `-N` before the extension when `path` is already in `used_paths`.
fn deduplicate_path(path: &Path, used_paths: &HashSet<PathBuf>) -> PathBuf {
    if !used_paths.contains(path) {
        return path.to_path_buf();
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("attachment");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let parent = path.parent().unwrap_or(Path::new(""));
    for i in 2u32.. {
        let new_name = if ext.is_empty() {
            format!("{stem}-{i}")
        } else {
            format!("{stem}-{i}.{ext}")
        };
        let candidate = parent.join(&new_name);
        if !used_paths.contains(&candidate) {
            return candidate;
        }
    }
    path.to_path_buf()
}

fn attachment_filename(attachment: &JiraAttachment) -> &str {
    attachment
        .filename
        .as_deref()
        .and_then(|filename| Path::new(filename).file_name()?.to_str())
        .or(attachment.id.as_deref())
        .unwrap_or("attachment")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_jira_attachment_metadata() {
        let attachment: JiraAttachment = serde_json::from_value(serde_json::json!({
            "id": "10042",
            "filename": "error-screenshot.png",
            "mimeType": "image/png",
            "size": 2048,
            "created": "2026-05-18T10:00:00.000+0000",
            "content": "https://example.atlassian.net/rest/api/3/attachment/content/10042",
            "author": {
                "accountId": "abc",
                "displayName": "Neo",
                "active": true
            }
        }))
        .expect("attachment metadata");

        assert_eq!(attachment.id.as_deref(), Some("10042"));
        assert_eq!(attachment.filename.as_deref(), Some("error-screenshot.png"));
        assert_eq!(attachment.mime_type.as_deref(), Some("image/png"));
        assert_eq!(attachment.size, Some(2048));
        assert_eq!(
            attachment
                .author
                .as_ref()
                .and_then(|author| author.display_name.as_deref()),
            Some("Neo")
        );
    }

    #[test]
    fn builds_attachment_output_paths() {
        assert_eq!(
            attachment_output_path(None, "error.png"),
            PathBuf::from("error.png")
        );
        assert_eq!(
            attachment_output_path(Some(Path::new("downloaded.png")), "error.png"),
            PathBuf::from("downloaded.png")
        );
    }

    #[test]
    fn attachment_filename_uses_basename() {
        let attachment = JiraAttachment {
            id: Some("10042".to_owned()),
            filename: Some("../error.png".to_owned()),
            mime_type: None,
            size: None,
            author: None,
            created: None,
            content: None,
            thumbnail: None,
        };

        assert_eq!(attachment_filename(&attachment), "error.png");
    }
}
