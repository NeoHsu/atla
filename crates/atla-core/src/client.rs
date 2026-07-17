use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use serde::{Deserialize, Serialize};

use crate::{AtlassianProduct, HttpPolicy, Profile};

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
    policy: HttpPolicy,
    max_pages: Option<u32>,
    max_items: Option<u32>,
    pages_fetched: Arc<AtomicU32>,
    verbose: bool,
}

impl AtlassianClient {
    pub fn new(
        instance: AtlassianInstance,
        email: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self::new_with_policy(instance, email, token, HttpPolicy::default())
    }

    pub fn new_with_policy(
        instance: AtlassianInstance,
        email: impl Into<String>,
        token: impl Into<String>,
        policy: HttpPolicy,
    ) -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(policy.connect_timeout)
            .build()
            .expect("reqwest client construction only fails on TLS backend init");
        Self {
            http,
            instance,
            email: email.into(),
            token: token.into(),
            policy,
            max_pages: None,
            max_items: None,
            pages_fetched: Arc::new(AtomicU32::new(0)),
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_execution_limits(mut self, max_pages: Option<u32>, max_items: Option<u32>) -> Self {
        self.max_pages = max_pages;
        self.max_items = max_items;
        self.pages_fetched.store(0, Ordering::Relaxed);
        self
    }

    pub fn from_profile(profile: &Profile, token: impl Into<String>) -> Self {
        Self::new(
            AtlassianInstance::new(&profile.instance),
            profile.email.clone(),
            token,
        )
    }

    pub fn from_profile_for_product(
        profile: &Profile,
        token: impl Into<String>,
        product: AtlassianProduct,
    ) -> Self {
        Self::from_profile_for_product_with_policy(profile, token, product, HttpPolicy::default())
    }

    pub fn from_profile_for_product_with_policy(
        profile: &Profile,
        token: impl Into<String>,
        product: AtlassianProduct,
        policy: HttpPolicy,
    ) -> Self {
        Self::new_with_policy(
            AtlassianInstance::new(profile.api_base_url(product)),
            profile.email.clone(),
            token,
            policy,
        )
    }

    pub(crate) fn take_page(&self) -> bool {
        let Some(max_pages) = self.max_pages else {
            return true;
        };
        self.pages_fetched
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (current < max_pages).then_some(current + 1)
            })
            .is_ok()
    }

    pub(crate) fn effective_item_limit(&self, requested: u32) -> u32 {
        self.max_items
            .map_or(requested.max(1), |maximum| requested.max(1).min(maximum))
    }

    pub fn instance(&self) -> &AtlassianInstance {
        &self.instance
    }

    fn log_request(&self, method: &str, url: &str) {
        if self.verbose {
            eprintln!("[verbose] {} {}", method, url);
        }
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::GET, path, self.policy.request_timeout)
            .header(reqwest::header::ACCEPT, "application/json")
    }

    /// Like `get()` but without forcing `Accept: application/json` — use for binary downloads.
    pub fn get_binary(&self, path: &str) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::GET, path, self.policy.transfer_timeout)
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::POST, path, self.policy.request_timeout)
            .header(reqwest::header::ACCEPT, "application/json")
    }

    pub fn put(&self, path: &str) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::PUT, path, self.policy.request_timeout)
            .header(reqwest::header::ACCEPT, "application/json")
    }

    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::DELETE, path, self.policy.request_timeout)
            .header(reqwest::header::ACCEPT, "application/json")
    }

    pub fn post_multipart(&self, path: &str) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::POST, path, self.policy.transfer_timeout)
            .header(reqwest::header::ACCEPT, "application/json")
            .header("X-Atlassian-Token", "no-check")
    }

    fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        timeout: std::time::Duration,
    ) -> reqwest::RequestBuilder {
        let url = self.url(path);
        self.log_request(method.as_str(), &url);
        let request = self.http.request(method, &url).timeout(timeout);
        if self.is_same_origin(&url) {
            request.basic_auth(&self.email, Some(&self.token))
        } else {
            request
        }
    }

    fn is_same_origin(&self, url: &str) -> bool {
        let Ok(base) = reqwest::Url::parse(&self.instance.base_url) else {
            return false;
        };
        let Ok(candidate) = reqwest::Url::parse(url) else {
            return false;
        };
        base.scheme() == candidate.scheme()
            && base.host_str() == candidate.host_str()
            && base.port_or_known_default() == candidate.port_or_known_default()
    }

    /// A reqwest client with Basic auth in its default headers, shared by the
    /// progenitor-generated API clients.
    pub(crate) fn authed_http_client(&self) -> reqwest::Client {
        use base64::Engine;
        use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

        let creds = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", self.email, self.token));
        let mut value = HeaderValue::from_str(&format!("Basic {creds}"))
            .expect("base64 credentials are always a valid header value");
        value.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, value);
        let retry_host = reqwest::Url::parse(&self.instance.base_url)
            .ok()
            .and_then(|url| url.host_str().map(str::to_owned))
            .unwrap_or_default();
        let retry_policy = reqwest::retry::for_host(retry_host)
            .max_retries_per_request(MAX_RETRIES)
            .classify_fn(|attempt| {
                let idempotent = matches!(
                    attempt.method().as_str(),
                    "GET" | "HEAD" | "PUT" | "DELETE" | "OPTIONS" | "TRACE"
                );
                match attempt.status().map(|status| status.as_u16()) {
                    Some(429) => attempt.retryable(),
                    Some(502..=504) if idempotent => attempt.retryable(),
                    None if idempotent && attempt.error().is_some() => attempt.retryable(),
                    _ => attempt.success(),
                }
            });
        reqwest::Client::builder()
            .connect_timeout(self.policy.connect_timeout)
            .timeout(self.policy.request_timeout)
            .retry(retry_policy)
            .default_headers(headers)
            .build()
            .expect("reqwest client construction only fails on TLS backend init")
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
    #[error("network error: {0}")]
    Network(String),
    #[error("Atlassian API returned {status}: {body}")]
    Http {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("{method} mutation outcome is unknown: {message}; verify remote state before retrying")]
    AmbiguousMutation {
        method: reqwest::Method,
        status: Option<reqwest::StatusCode>,
        message: String,
    },
    #[error("{0}")]
    Io(String),
}

impl ApiError {
    /// Whether retrying the same request may succeed (transient network
    /// failures, rate limiting, or server-side errors).
    pub fn retryable(&self) -> bool {
        match self {
            ApiError::Network(_) => true,
            ApiError::Request(e) => e.is_timeout() || e.is_connect(),
            ApiError::Http { status, .. } => {
                *status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
            }
            ApiError::AmbiguousMutation { .. } | ApiError::Decode(_) | ApiError::Io(_) => false,
        }
    }

    /// HTTP status of the failed request, when one was received.
    pub fn status(&self) -> Option<reqwest::StatusCode> {
        match self {
            ApiError::Http { status, .. } => Some(*status),
            ApiError::AmbiguousMutation { status, .. } => *status,
            ApiError::Request(e) => e.status(),
            _ => None,
        }
    }
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
                .map(|(key, value)| {
                    format!("{}: {}", key, value.as_str().unwrap_or(&value.to_string()))
                })
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
        format!("{}…", &body[..body.floor_char_boundary(500)])
    } else {
        body.to_owned()
    }
}

const MAX_RETRIES: u32 = 2;
const MAX_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(30);

fn is_idempotent(method: &reqwest::Method) -> bool {
    matches!(
        *method,
        reqwest::Method::GET
            | reqwest::Method::HEAD
            | reqwest::Method::PUT
            | reqwest::Method::DELETE
            | reqwest::Method::OPTIONS
            | reqwest::Method::TRACE
    )
}

fn should_retry(
    method: &reqwest::Method,
    outcome: &Result<reqwest::Response, reqwest::Error>,
) -> bool {
    match outcome {
        Ok(response) if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS => true,
        Ok(response) => {
            is_idempotent(method)
                && matches!(
                    response.status(),
                    reqwest::StatusCode::BAD_GATEWAY
                        | reqwest::StatusCode::SERVICE_UNAVAILABLE
                        | reqwest::StatusCode::GATEWAY_TIMEOUT
                )
        }
        Err(error) => is_idempotent(method) && (error.is_connect() || error.is_timeout()),
    }
}

fn parse_retry_after(value: &str, now: std::time::SystemTime) -> Option<std::time::Duration> {
    value
        .parse::<u64>()
        .ok()
        .map(std::time::Duration::from_secs)
        .or_else(|| {
            httpdate::parse_http_date(value)
                .ok()?
                .duration_since(now)
                .ok()
        })
}

fn retry_delay(response: Option<&reqwest::Response>, attempt: u32) -> std::time::Duration {
    let retry_after = response
        .and_then(|response| response.headers().get(reqwest::header::RETRY_AFTER))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| parse_retry_after(value, std::time::SystemTime::now()));
    retry_after
        .unwrap_or_else(|| std::time::Duration::from_millis(500 << attempt))
        .min(MAX_RETRY_DELAY)
}

fn request_failure(method: &reqwest::Method, error: reqwest::Error) -> ApiError {
    if !is_idempotent(method) && error.is_timeout() {
        ApiError::AmbiguousMutation {
            method: method.clone(),
            status: error.status(),
            message: error.to_string(),
        }
    } else {
        ApiError::Request(error)
    }
}

fn response_failure(
    method: reqwest::Method,
    status: reqwest::StatusCode,
    body: String,
) -> ApiError {
    let body = extract_api_error_body(&body);
    if !is_idempotent(&method)
        && status.is_server_error()
        && status != reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        ApiError::AmbiguousMutation {
            method,
            status: Some(status),
            message: format!("Atlassian API returned {status}: {body}"),
        }
    } else {
        ApiError::Http { status, body }
    }
}

/// Sends a request, retrying 429 responses and retrying other transient
/// failures only for idempotent methods. Non-cloneable bodies are sent once.
async fn send_with_retry(
    request: reqwest::RequestBuilder,
) -> Result<(reqwest::Response, reqwest::Method), ApiError> {
    let (client, request) = request.build_split();
    let request = request?;
    let method = request.method().clone();
    let retry_template = request.try_clone();
    let mut first_attempt = Some(request);
    let mut attempt = 0u32;

    loop {
        let this_attempt = match first_attempt.take() {
            Some(request) => request,
            None => retry_template
                .as_ref()
                .and_then(reqwest::Request::try_clone)
                .expect("retry template was checked before retrying"),
        };
        let outcome = client.execute(this_attempt).await;
        let can_retry =
            retry_template.is_some() && should_retry(&method, &outcome) && attempt < MAX_RETRIES;
        if !can_retry {
            return match outcome {
                Ok(response) => Ok((response, method)),
                Err(error) => Err(request_failure(&method, error)),
            };
        }
        tokio::time::sleep(retry_delay(outcome.as_ref().ok(), attempt)).await;
        attempt += 1;
    }
}

pub async fn read_json<T: serde::de::DeserializeOwned>(
    request: reqwest::RequestBuilder,
) -> Result<T, ApiError> {
    let (response, method) = send_with_retry(request).await?;
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(response_failure(method, status, body));
    }

    response.json::<T>().await.map_err(ApiError::Request)
}

pub async fn read_empty(request: reqwest::RequestBuilder) -> Result<(), ApiError> {
    let (response, method) = send_with_retry(request).await?;
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(response_failure(method, status, body));
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
    fn absolute_cross_origin_urls_do_not_receive_credentials() {
        let client = AtlassianClient::new(
            AtlassianInstance::new("https://example.atlassian.net/"),
            "neo@example.com",
            "token",
        );

        let same_origin = client
            .get("https://example.atlassian.net/rest/api/3/myself")
            .build()
            .expect("same-origin request should build");
        let cross_origin = client
            .get("https://api.media.atlassian.com/file/123")
            .build()
            .expect("cross-origin request should build");
        let downgrade = client
            .get("http://example.atlassian.net/rest/api/3/myself")
            .build()
            .expect("downgrade request should build");

        assert!(
            same_origin
                .headers()
                .contains_key(reqwest::header::AUTHORIZATION)
        );
        assert!(
            !cross_origin
                .headers()
                .contains_key(reqwest::header::AUTHORIZATION)
        );
        assert!(
            !downgrade
                .headers()
                .contains_key(reqwest::header::AUTHORIZATION)
        );
    }

    #[tokio::test]
    async fn cross_origin_redirects_do_not_forward_credentials() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let destination = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/target"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&destination)
            .await;
        let source = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/redirect"))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("Location", format!("{}/target", destination.uri())),
            )
            .mount(&source)
            .await;
        let client = AtlassianClient::new(
            AtlassianInstance::new(source.uri()),
            "neo@example.com",
            "token",
        );

        client
            .get("/redirect")
            .send()
            .await
            .expect("redirect should succeed");

        let requests = destination
            .received_requests()
            .await
            .expect("request recording should be enabled");
        assert_eq!(requests.len(), 1);
        assert!(
            !requests[0]
                .headers
                .contains_key(reqwest::header::AUTHORIZATION)
        );
    }

    #[test]
    fn classifies_only_idempotent_methods_as_safe_for_transient_retries() {
        assert!(is_idempotent(&reqwest::Method::GET));
        assert!(is_idempotent(&reqwest::Method::PUT));
        assert!(is_idempotent(&reqwest::Method::DELETE));
        assert!(!is_idempotent(&reqwest::Method::POST));
        assert!(!is_idempotent(&reqwest::Method::PATCH));
    }

    #[test]
    fn parses_retry_after_seconds_and_http_dates() {
        use std::time::{Duration, SystemTime};

        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let later = now + Duration::from_secs(12);

        assert_eq!(parse_retry_after("7", now), Some(Duration::from_secs(7)));
        assert_eq!(
            parse_retry_after(&httpdate::fmt_http_date(later), now),
            Some(Duration::from_secs(12))
        );
        assert_eq!(parse_retry_after("not-a-date", now), None);
    }

    #[tokio::test]
    async fn non_idempotent_server_error_is_not_retried() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mutation"))
            .respond_with(ResponseTemplate::new(503))
            .expect(1)
            .mount(&server)
            .await;
        let client = AtlassianClient::new(
            AtlassianInstance::new(server.uri()),
            "neo@example.com",
            "token",
        );

        let error = read_empty(client.post("/mutation"))
            .await
            .expect_err("503 should fail");

        assert_eq!(
            error.status(),
            Some(reqwest::StatusCode::SERVICE_UNAVAILABLE)
        );
        assert!(!error.retryable());
        assert!(matches!(error, ApiError::AmbiguousMutation { .. }));
        server.verify().await;
    }

    #[tokio::test]
    async fn mutation_timeout_is_reported_as_ambiguous() {
        use std::time::Duration;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/slow-mutation"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(1)))
            .expect(1)
            .mount(&server)
            .await;
        let policy = HttpPolicy::default().with_request_timeout(Duration::from_millis(20));
        let client = AtlassianClient::new_with_policy(
            AtlassianInstance::new(server.uri()),
            "neo@example.com",
            "token",
            policy,
        );

        let error = read_empty(client.post("/slow-mutation"))
            .await
            .expect_err("slow mutation should time out");

        assert!(!error.retryable());
        assert!(matches!(error, ApiError::AmbiguousMutation { .. }));
        server.verify().await;
    }

    #[tokio::test]
    async fn request_timeout_is_enforced() {
        use std::time::Duration;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/slow"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(1)))
            .mount(&server)
            .await;
        let policy = HttpPolicy::default().with_request_timeout(Duration::from_millis(20));
        let client = AtlassianClient::new_with_policy(
            AtlassianInstance::new(server.uri()),
            "neo@example.com",
            "token",
            policy,
        );

        let error = client
            .get("/slow")
            .send()
            .await
            .expect_err("slow request should time out");

        assert!(error.is_timeout());
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
