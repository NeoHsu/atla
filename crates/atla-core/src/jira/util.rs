use atla_jira_api::apis as generated_apis;
use serde::Serialize;
use std::collections::HashMap;

use crate::client::ApiError;
use crate::markdown::markdown_to_adf;

pub(super) fn limit_i32(limit: u32) -> i32 {
    limit.min(i32::MAX as u32) as i32
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

pub(super) fn issue_fields(fields: Option<&[String]>) -> Vec<String> {
    fields
        .filter(|fields| !fields.is_empty())
        .map(|fields| fields.to_vec())
        .unwrap_or_else(super::default_issue_fields)
}

pub(super) fn serialized_string<T: Serialize>(value: T) -> Option<String> {
    match serde_json::to_value(value).ok()? {
        serde_json::Value::String(value) => Some(value),
        _ => None,
    }
}

pub(super) fn adf_body(text: &str) -> HashMap<String, serde_json::Value> {
    markdown_to_adf(text)
        .as_object()
        .expect("ADF root is an object")
        .clone()
        .into_iter()
        .collect()
}

pub(super) fn adf_plain_text(body: &HashMap<String, serde_json::Value>) -> String {
    let value = serde_json::Value::Object(body.clone().into_iter().collect());
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
