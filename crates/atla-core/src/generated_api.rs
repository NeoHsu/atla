//! Shared error mapping for the progenitor-generated API clients.
//!
//! All three generated crates re-export the same `progenitor_client::Error`
//! and `ResponseValue` types, so one mapping serves Jira, Confluence v2, and
//! Confluence v1. Always route errors through the body-reading path
//! ([`ProgenitorResultExt::or_api_error`] or [`generated_error_with_body`]);
//! the sync fallback cannot read the response body and loses the API's own
//! explanation of what went wrong.

use progenitor_client::Error;

use crate::client::{ApiError, extract_api_error_body};

/// Sync fallback for error variants that carry no readable body.
pub(crate) fn generated_error(error: Error<()>) -> ApiError {
    match error {
        Error::InvalidRequest(msg) => ApiError::Decode(msg),
        Error::CommunicationError(e) => ApiError::Network(e.to_string()),
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
        _ => ApiError::Decode("unknown API error".to_owned()),
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

/// Converts a generated-client result into an [`ApiError`] result, reading the
/// error response body so API messages (e.g. JQL syntax errors) reach users.
pub(crate) trait ProgenitorResultExt<T>: Sized {
    async fn or_api_error(self) -> Result<T, ApiError>;
}

impl<T> ProgenitorResultExt<T> for Result<T, Error<()>> {
    async fn or_api_error(self) -> Result<T, ApiError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(generated_error_with_body(error).await),
        }
    }
}
