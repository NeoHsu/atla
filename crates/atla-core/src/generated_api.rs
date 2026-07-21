//! Shared error mapping for the progenitor-generated API clients.
//!
//! All three generated crates re-export the same `progenitor_client::Error`
//! and `ResponseValue` types, so one mapping serves Jira, Confluence v2, and
//! Confluence v1. Always route calls through [`generated_request`]; its final
//! error path uses [`generated_error_with_body`] so the API's own explanation
//! reaches users. Generated retries share the raw-client backoff and
//! `Retry-After` behavior without retrying ambiguous mutations.

use progenitor_client::Error;

use crate::client::{ApiError, extract_api_error_body};

/// Sync fallback for error variants that carry no readable body.
pub(crate) fn generated_error(error: Error<()>) -> ApiError {
    match error {
        Error::InvalidRequest(msg) => ApiError::Decode(msg),
        Error::CommunicationError(e) | Error::InvalidUpgrade(e) | Error::ResponseBodyError(e) => {
            ApiError::Network(e.to_string())
        }
        Error::ErrorResponse(rv) => {
            let status = rv.status();
            ApiError::Http {
                status,
                body: format!("{:?}", rv.into_inner()),
            }
        }
        Error::InvalidResponsePayload(_, e) => ApiError::Decode(e.to_string()),
        Error::UnexpectedResponse(resp) => ApiError::Http {
            status: resp.status(),
            body: String::new(),
        },
        Error::Custom(message) => ApiError::Decode(message),
    }
}

pub(crate) async fn generated_error_with_body(error: Error<()>) -> ApiError {
    match error {
        Error::UnexpectedResponse(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            ApiError::Http {
                status,
                body: extract_api_error_body(&body),
            }
        }
        other => generated_error(other),
    }
}

fn response_headers(error: &Error<()>) -> Option<&reqwest::header::HeaderMap> {
    match error {
        Error::ErrorResponse(response) => Some(response.headers()),
        Error::UnexpectedResponse(response) => Some(response.headers()),
        _ => None,
    }
}

fn should_retry(method: &reqwest::Method, error: &Error<()>) -> bool {
    match error.status().map(|status| status.as_u16()) {
        Some(429) => true,
        Some(502..=504) => crate::client::is_idempotent(method),
        None => {
            crate::client::is_idempotent(method) && matches!(error, Error::CommunicationError(_))
        }
        _ => false,
    }
}

/// Executes a generated-client request with the shared bounded retry policy.
///
/// A received 429 is safe to repeat because the server explicitly rejected the
/// attempt. Other transient responses and transport failures are retried only
/// for idempotent methods; mutation failures remain ambiguous and fail closed.
pub(crate) async fn generated_request<T, F, Fut>(
    method: reqwest::Method,
    mut send: F,
) -> Result<T, ApiError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error<()>>>,
{
    let mut attempt = 0;
    loop {
        match send().await {
            Ok(value) => return Ok(value),
            Err(error) if attempt < crate::client::MAX_RETRIES && should_retry(&method, &error) => {
                let delay =
                    crate::client::retry_delay_from_headers(response_headers(&error), attempt);
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(error) => return Err(generated_error_with_body(error).await),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retryable_error(status: u16, retry_after: Option<&str>) -> Error<()> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(retry_after) = retry_after {
            headers.insert(
                reqwest::header::RETRY_AFTER,
                retry_after.parse().expect("valid Retry-After header"),
            );
        }
        Error::ErrorResponse(progenitor_client::ResponseValue::new(
            (),
            reqwest::StatusCode::from_u16(status).expect("valid test status"),
            headers,
        ))
    }

    #[test]
    fn generated_calls_use_shared_retry_wrapper() {
        let modules = [
            (
                "confluence/attachments.rs",
                include_str!("confluence/attachments.rs"),
            ),
            ("confluence/blog.rs", include_str!("confluence/blog.rs")),
            (
                "confluence/comments.rs",
                include_str!("confluence/comments.rs"),
            ),
            ("confluence/labels.rs", include_str!("confluence/labels.rs")),
            ("confluence/pages.rs", include_str!("confluence/pages.rs")),
            ("confluence/search.rs", include_str!("confluence/search.rs")),
            ("confluence/spaces.rs", include_str!("confluence/spaces.rs")),
            ("jira/comments.rs", include_str!("jira/comments.rs")),
            ("jira/issues.rs", include_str!("jira/issues.rs")),
            ("jira/links.rs", include_str!("jira/links.rs")),
            ("jira/projects.rs", include_str!("jira/projects.rs")),
        ];

        for (path, source) in modules {
            let mut ranges = Vec::new();
            let mut search_from = 0;
            while let Some(relative_start) = source[search_from..].find("generated_request(") {
                let start = search_from + relative_start;
                let open = start + "generated_request".len();
                let mut depth = 0_u32;
                let mut end = None;
                for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
                    match byte {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                end = Some(open + offset + 1);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let end = end.expect("generated_request call should have balanced parentheses");
                ranges.push(start..end);
                search_from = end;
            }

            for (index, _) in source.match_indices("self.generated") {
                assert!(
                    ranges.iter().any(|range| range.contains(&index)),
                    "{path} accesses a generated client outside generated_request near byte {index}"
                );
            }
        }
    }

    #[tokio::test]
    async fn retries_idempotent_transient_response_with_retry_after() {
        let mut calls = 0;
        let value = generated_request(reqwest::Method::GET, || {
            calls += 1;
            std::future::ready(if calls == 1 {
                Err(retryable_error(503, Some("0")))
            } else {
                Ok(42)
            })
        })
        .await
        .expect("GET should retry");

        assert_eq!(value, 42);
        assert_eq!(calls, 2);
    }

    #[tokio::test]
    async fn does_not_retry_ambiguous_mutation_failure() {
        let mut calls = 0;
        let error = generated_request(reqwest::Method::POST, || {
            calls += 1;
            std::future::ready::<Result<(), Error<()>>>(Err(retryable_error(503, Some("0"))))
        })
        .await
        .expect_err("POST 503 must fail closed");

        assert_eq!(calls, 1);
        assert!(matches!(
            error,
            ApiError::Http { status, .. }
                if status == reqwest::StatusCode::SERVICE_UNAVAILABLE
        ));
    }

    #[tokio::test]
    async fn retries_explicit_rate_limit_for_mutation() {
        let mut calls = 0;
        let value = generated_request(reqwest::Method::POST, || {
            calls += 1;
            std::future::ready(if calls == 1 {
                Err(retryable_error(429, Some("0")))
            } else {
                Ok(7)
            })
        })
        .await
        .expect("429 explicitly rejects the mutation attempt");

        assert_eq!(value, 7);
        assert_eq!(calls, 2);
    }
}
