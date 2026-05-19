use atla_confluence_api::apis as generated_apis;
use atla_confluence_v1_api::apis as generated_v1_apis;

use crate::client::ApiError;

pub(super) fn limit_i32(limit: u32) -> i32 {
    limit.min(i32::MAX as u32) as i32
}

pub(super) fn parse_i64_id(id: &str) -> Result<i64, ApiError> {
    id.parse()
        .map_err(|_| ApiError::Decode(format!("expected numeric Confluence id, got `{id}`")))
}

pub(super) fn optional_i64_vec(id: Option<&str>) -> Result<Option<Vec<i64>>, ApiError> {
    id.map(|id| parse_i64_id(id).map(|id| vec![id])).transpose()
}

pub(super) fn generated_error<T>(error: generated_apis::Error<T>) -> ApiError {
    match error {
        generated_apis::Error::Reqwest(error) => ApiError::Decode(error.to_string()),
        generated_apis::Error::Serde(error) => ApiError::Decode(error.to_string()),
        generated_apis::Error::Io(error) => ApiError::Decode(error.to_string()),
        generated_apis::Error::ResponseError(response) => ApiError::Http {
            status: response.status,
            body: crate::client::extract_api_error_body(&response.content),
        },
    }
}

pub(super) fn generated_v1_error<T>(error: generated_v1_apis::Error<T>) -> ApiError {
    match error {
        generated_v1_apis::Error::Reqwest(error) => ApiError::Decode(error.to_string()),
        generated_v1_apis::Error::Serde(error) => ApiError::Decode(error.to_string()),
        generated_v1_apis::Error::Io(error) => ApiError::Decode(error.to_string()),
        generated_v1_apis::Error::ResponseError(response) => ApiError::Http {
            status: response.status,
            body: crate::client::extract_api_error_body(&response.content),
        },
    }
}
