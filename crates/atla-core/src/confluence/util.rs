use std::num::NonZeroU32;

use crate::client::ApiError;

pub(super) use crate::generated_api::generated_request;

/// Per-request page cap used when iterating Confluence v2 cursor endpoints.
/// The server enforces its own ceiling (250 for most list endpoints); we stay
/// well under it so a single user `--limit` translates into predictable batches.
pub(super) const CONFLUENCE_LIST_PAGE_CAP: u32 = 100;

/// Extracts the `cursor` query parameter from a `_links.next` URL returned by
/// Confluence v2. The link may be relative (`/wiki/api/v2/...`) or absolute;
/// we parse just the query string and pick the `cursor=...` value.
pub(super) fn cursor_from_next_link(next: &str) -> Option<String> {
    let query = next.split_once('?').map(|(_, q)| q).unwrap_or(next);
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("cursor=") {
            return Some(percent_decode(value));
        }
    }
    None
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(super) fn limit_i32(limit: u32) -> i32 {
    limit.min(i32::MAX as u32) as i32
}

pub(super) fn limit_non_zero(limit: u32) -> Result<NonZeroU32, ApiError> {
    NonZeroU32::new(limit).ok_or_else(|| ApiError::Decode("limit must be at least 1".to_owned()))
}

pub(super) fn parse_i64_id(id: &str) -> Result<i64, ApiError> {
    let numeric = id.strip_prefix("att").unwrap_or(id);
    numeric
        .parse()
        .map_err(|_| ApiError::Decode(format!("expected numeric Confluence id, got `{id}`")))
}

pub(super) fn optional_i64_vec(id: Option<&str>) -> Result<Option<Vec<i64>>, ApiError> {
    id.map(|id| parse_i64_id(id).map(|id| vec![id])).transpose()
}
