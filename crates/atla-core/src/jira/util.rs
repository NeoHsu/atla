use crate::markdown::markdown_to_adf;

pub(super) use crate::generated_api::{ProgenitorResultExt, generated_error_with_body};

pub(super) fn limit_i32(limit: u32) -> i32 {
    limit.min(i32::MAX as u32) as i32
}

/// Per-request page cap for Jira list endpoints that share the classic
/// startAt/maxResults model. Larger user limits are reached by paginating.
pub(super) const JIRA_LIST_PAGE_CAP: u32 = 100;

/// Decides whether to issue another offset-paginated request.
///
/// Stops when no items were returned, when the server flagged this as the
/// last page, when we have collected enough to satisfy `max_results`, or when
/// `total` has been reached. Returns the next `start_at` to use otherwise.
pub(super) fn next_offset(
    collected: u64,
    max_results: u64,
    received: u64,
    is_last: Option<bool>,
    total: Option<u64>,
    start_at: u64,
) -> Option<u64> {
    if received == 0 || collected >= max_results || matches!(is_last, Some(true)) {
        return None;
    }
    let next = start_at.checked_add(received)?;
    if total.is_some_and(|total| next >= total) {
        return None;
    }
    Some(next)
}

pub(super) fn issue_fields(fields: Option<&[String]>) -> Vec<String> {
    fields
        .filter(|fields| !fields.is_empty())
        .map(|fields| fields.to_vec())
        .unwrap_or_else(super::default_issue_fields)
}

pub(super) fn adf_body(text: &str) -> serde_json::Map<String, serde_json::Value> {
    markdown_to_adf(text)
        .as_object()
        .expect("ADF root is an object")
        .clone()
}

pub(super) fn adf_plain_text(body: &serde_json::Map<String, serde_json::Value>) -> String {
    let value = serde_json::Value::Object(body.clone());
    let mut parts = Vec::new();
    collect_adf_text(&value, &mut parts);
    parts.join("").trim().to_owned()
}

fn collect_adf_text(value: &serde_json::Value, parts: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(object) => {
            let node_type = object
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if node_type == "hardBreak" {
                parts.push("\n".to_owned());
            } else if let Some(text) = object.get("text").and_then(serde_json::Value::as_str) {
                parts.push(text.to_owned());
            } else if let Some(text) = object
                .get("attrs")
                .and_then(serde_json::Value::as_object)
                .and_then(|attributes| {
                    attributes
                        .get("text")
                        .and_then(serde_json::Value::as_str)
                        .or_else(|| attributes.get("label").and_then(serde_json::Value::as_str))
                        .or_else(|| attributes.get("url").and_then(serde_json::Value::as_str))
                })
            {
                parts.push(text.to_owned());
            }
            if let Some(content) = object.get("content") {
                collect_adf_text(content, parts);
                if matches!(
                    node_type,
                    "paragraph"
                        | "heading"
                        | "listItem"
                        | "bulletList"
                        | "orderedList"
                        | "blockquote"
                        | "codeBlock"
                        | "rule"
                        | "panel"
                ) {
                    parts.push("\n".to_owned());
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_adf_text(item, parts);
            }
        }
        _ => {}
    }
}

pub(super) fn quote_jql_value(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
