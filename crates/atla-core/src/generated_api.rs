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

    fn rust_source_files(root: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
        let mut pending = vec![root.to_path_buf()];
        let mut files = Vec::new();
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(directory)? {
                let entry = entry?;
                let path = entry.path();
                let file_type = entry.file_type()?;
                if file_type.is_dir() {
                    pending.push(path);
                } else if file_type.is_file()
                    && path
                        .extension()
                        .is_some_and(|extension| extension == std::ffi::OsStr::new("rs"))
                {
                    files.push(path);
                }
            }
        }
        files.sort();
        Ok(files)
    }

    #[test]
    fn generated_calls_use_shared_retry_wrapper() -> Result<(), Box<dyn std::error::Error>> {
        let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut modules = Vec::new();
        for product in ["confluence", "jira"] {
            for path in rust_source_files(&source_root.join(product))? {
                let source = std::fs::read_to_string(&path)?;
                if source.contains("self.generated") {
                    modules.push((path, source));
                }
            }
        }
        assert!(
            !modules.is_empty(),
            "expected to discover generated-client source modules"
        );

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
                let Some(end) = end else {
                    panic!(
                        "generated_request call in {} has unbalanced parentheses",
                        path.display()
                    );
                };
                ranges.push(start..end);
                search_from = end;
            }

            for (index, _) in source.match_indices("self.generated") {
                assert!(
                    ranges.iter().any(|range| range.contains(&index)),
                    "{} accesses a generated client outside generated_request near byte {index}",
                    path.display()
                );
            }
        }
        Ok(())
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
