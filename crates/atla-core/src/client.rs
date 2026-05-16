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
        }
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

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.http
            .get(self.url(path))
            .basic_auth(&self.email, Some(&self.token))
            .header(reqwest::header::ACCEPT, "application/json")
    }

    pub fn url(&self, path: &str) -> String {
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
    #[error("Atlassian API returned {status}: {body}")]
    Http {
        status: reqwest::StatusCode,
        body: String,
    },
}

pub async fn read_json<T: serde::de::DeserializeOwned>(
    request: reqwest::RequestBuilder,
) -> Result<T, ApiError> {
    let response = request.send().await?;
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ApiError::Http { status, body });
    }

    response.json::<T>().await.map_err(ApiError::Request)
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
}
