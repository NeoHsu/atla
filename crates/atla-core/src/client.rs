use serde::{Deserialize, Serialize};

use crate::Profile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlassianInstance {
    pub base_url: String,
}

impl AtlassianInstance {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtlassianClient {
    http: reqwest::Client,
    instance: AtlassianInstance,
    email: String,
    token: String,
    verbose: bool,
}

impl AtlassianClient {
    pub fn new(
        instance: AtlassianInstance,
        email: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            instance,
            email: email.into(),
            token: token.into(),
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn from_profile(profile: &Profile, token: impl Into<String>) -> Self {
        Self::new(
            AtlassianInstance::new(&profile.instance),
            profile.email.clone(),
            token,
        )
    }

    pub fn instance(&self) -> &AtlassianInstance {
        &self.instance
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }

    pub(crate) fn token(&self) -> &str {
        &self.token
    }

    fn log_request(&self, method: &str, url: &str) {
        if self.verbose {
            eprintln!("[verbose] {} {}", method, url);
        }
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let url = self.url(path);
        self.log_request("GET", &url);
        self.http
            .get(url)
            .basic_auth(&self.email, Some(&self.token))
            .header(reqwest::header::ACCEPT, "application/json")
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = self.url(path);
        self.log_request("POST", &url);
        self.http
            .post(url)
            .basic_auth(&self.email, Some(&self.token))
            .header(reqwest::header::ACCEPT, "application/json")
    }

    pub fn put(&self, path: &str) -> reqwest::RequestBuilder {
        let url = self.url(path);
        self.log_request("PUT", &url);
        self.http
            .put(url)
            .basic_auth(&self.email, Some(&self.token))
            .header(reqwest::header::ACCEPT, "application/json")
    }

    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        let url = self.url(path);
        self.log_request("DELETE", &url);
        self.http
            .delete(url)
            .basic_auth(&self.email, Some(&self.token))
            .header(reqwest::header::ACCEPT, "application/json")
    }

    pub fn post_multipart(&self, path: &str) -> reqwest::RequestBuilder {
        let url = self.url(path);
        self.log_request("POST", &url);
        self.http
            .post(url)
            .basic_auth(&self.email, Some(&self.token))
            .header(reqwest::header::ACCEPT, "application/json")
            .header("X-Atlassian-Token", "no-check")
    }

    pub fn url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_owned();
        }
        format!(
            "{}/{}",
            self.instance.base_url,
            path.trim_start_matches('/')
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("failed to decode response: {0}")]
    Decode(String),
    #[error("Atlassian API returned {status}: {body}")]
    Http {
        status: reqwest::StatusCode,
        body: String,
    },
}

pub(crate) fn extract_api_error_body(body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msgs) = json["errorMessages"].as_array() {
            let msgs: Vec<&str> = msgs.iter().filter_map(|message| message.as_str()).collect();
            if !msgs.is_empty() {
                return msgs.join("; ");
            }
        }

        if let Some(errors) = json["errors"].as_object() {
            let msgs: Vec<String> = errors
                .iter()
                .map(|(key, value)| format!("{}: {}", key, value.as_str().unwrap_or(&value.to_string())))
                .collect();
            if !msgs.is_empty() {
                return msgs.join("; ");
            }
        }

        if let Some(msg) = json["message"].as_str() {
            return msg.to_owned();
        }

        if let Some(msg) = json["error"].as_str() {
            return msg.to_owned();
        }
    }

    if body.len() > 500 {
        format!("{}…", &body[..500])
    } else {
        body.to_owned()
    }
}

pub async fn read_json<T: serde::de::DeserializeOwned>(
    request: reqwest::RequestBuilder,
) -> Result<T, ApiError> {
    let response = request.send().await?;
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ApiError::Http {
            status,
            body: extract_api_error_body(&body),
        });
    }

    response.json::<T>().await.map_err(ApiError::Request)
}

pub async fn read_empty(request: reqwest::RequestBuilder) -> Result<(), ApiError> {
    let response = request.send().await?;
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ApiError::Http {
            status,
            body: extract_api_error_body(&body),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_base_url_and_paths() {
        let client = AtlassianClient::new(
            AtlassianInstance::new("https://example.atlassian.net/"),
            "neo@example.com",
            "token",
        );

        assert_eq!(
            client.url("/rest/api/3/project/search"),
            "https://example.atlassian.net/rest/api/3/project/search"
        );
    }

    #[test]
    fn leaves_absolute_urls_unchanged() {
        let client = AtlassianClient::new(
            AtlassianInstance::new("https://example.atlassian.net/"),
            "neo@example.com",
            "token",
        );

        assert_eq!(
            client.url("https://api.media.atlassian.com/file/123"),
            "https://api.media.atlassian.com/file/123"
        );
    }

    #[test]
    fn multipart_posts_include_atlassian_token_header() {
        let client = AtlassianClient::new(
            AtlassianInstance::new("https://example.atlassian.net/"),
            "neo@example.com",
            "token",
        );

        let request = client
            .post_multipart("/rest/api/3/issue/DEMO-1/attachments")
            .build()
            .expect("request should build");

        assert_eq!(
            request
                .headers()
                .get("X-Atlassian-Token")
                .and_then(|value| value.to_str().ok()),
            Some("no-check")
        );
        assert_eq!(
            request
                .headers()
                .get(reqwest::header::ACCEPT)
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
    }
}
