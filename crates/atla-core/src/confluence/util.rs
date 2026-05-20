use std::num::NonZeroU32;

use crate::client::ApiError;

pub(super) fn limit_i32(limit: u32) -> i32 {
    limit.min(i32::MAX as u32) as i32
}

pub(super) fn limit_non_zero(limit: u32) -> Result<NonZeroU32, ApiError> {
    NonZeroU32::new(limit).ok_or_else(|| ApiError::Decode("limit must be at least 1".to_owned()))
}

pub(super) fn parse_i64_id(id: &str) -> Result<i64, ApiError> {
    id.parse()
        .map_err(|_| ApiError::Decode(format!("expected numeric Confluence id, got `{id}`")))
}

pub(super) fn optional_i64_vec(id: Option<&str>) -> Result<Option<Vec<i64>>, ApiError> {
    id.map(|id| parse_i64_id(id).map(|id| vec![id])).transpose()
}

pub(super) fn generated_error(error: atla_confluence_api::Error<()>) -> ApiError {
    match error {
        atla_confluence_api::Error::InvalidRequest(msg) => ApiError::Decode(msg),
        atla_confluence_api::Error::CommunicationError(e) => ApiError::Decode(e.to_string()),
        atla_confluence_api::Error::ErrorResponse(rv) => {
            let status = rv.status();
            ApiError::Http {
                status,
                body: format!("{:?}", rv.into_inner()),
            }
        }
        atla_confluence_api::Error::InvalidResponsePayload(_, e) => ApiError::Decode(e.to_string()),
        atla_confluence_api::Error::UnexpectedResponse(resp) => ApiError::Http {
            status: resp.status(),
            body: String::new(),
        },
        _ => ApiError::Decode("unknown API error".to_owned()),
    }
}

pub(super) fn generated_v1_error(error: atla_confluence_v1_api::Error<()>) -> ApiError {
    match error {
        atla_confluence_v1_api::Error::InvalidRequest(msg) => ApiError::Decode(msg),
        atla_confluence_v1_api::Error::CommunicationError(e) => ApiError::Decode(e.to_string()),
        atla_confluence_v1_api::Error::ErrorResponse(rv) => {
            let status = rv.status();
            ApiError::Http {
                status,
                body: format!("{:?}", rv.into_inner()),
            }
        }
        atla_confluence_v1_api::Error::InvalidResponsePayload(_, e) => {
            ApiError::Decode(e.to_string())
        }
        atla_confluence_v1_api::Error::UnexpectedResponse(resp) => ApiError::Http {
            status: resp.status(),
            body: String::new(),
        },
        _ => ApiError::Decode("unknown API error".to_owned()),
    }
}
