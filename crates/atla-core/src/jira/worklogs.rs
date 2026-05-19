use atla_jira_api::apis as generated_apis;

use super::JiraClient;
use super::models::{JiraWorklog, JiraWorklogCreate, JiraWorklogPage};
use super::util::{generated_error, limit_i32};
use crate::client::ApiError;

impl JiraClient {
    pub async fn add_worklog(&self, worklog: &JiraWorklogCreate) -> Result<JiraWorklog, ApiError> {
        let normalized_time = normalize_jira_time(&worklog.time_spent);
        if !is_valid_jira_time_format(&normalized_time) {
            return Err(ApiError::Io(format!(
                "invalid time format `{}` — expected Jira duration notation, e.g. 1w, 2d, 3h, 30m or combinations like \"2h 30m\"",
                worklog.time_spent
            )));
        }

        let mut json = worklog.to_json();
        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                "timeSpent".to_owned(),
                serde_json::Value::String(normalized_time),
            );
        }

        let value = generated_apis::issue_worklogs_api::add_worklog(
            &self.generated,
            &worklog.issue_id_or_key,
            &json,
        )
        .await
        .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    pub async fn list_worklogs(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
    ) -> Result<JiraWorklogPage, ApiError> {
        let value = generated_apis::issue_worklogs_api::get_issue_worklog(
            &self.generated,
            issue_id_or_key,
            Some(0),
            Some(limit_i32(max_results)),
        )
        .await
        .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }
}

/// Normalizes compact Jira duration strings by inserting spaces between unit boundaries.
fn normalize_jira_time(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut chars = s.trim().chars().peekable();
    while let Some(ch) = chars.next() {
        result.push(ch);
        if matches!(ch, 'w' | 'd' | 'h' | 'm')
            && chars.peek().is_some_and(|next| next.is_ascii_digit())
        {
            result.push(' ');
        }
    }
    result
}

/// Validates that a time string matches Jira's duration notation.
fn is_valid_jira_time_format(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let valid_units = ['w', 'd', 'h', 'm'];
    let mut total_minutes: u64 = 0;
    for token in s.split_whitespace() {
        let last = token.chars().last();
        let unit = match last {
            Some(c) if valid_units.contains(&c) => c,
            _ => return false,
        };
        let digits = &token[..token.len() - unit.len_utf8()];
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        let n: u64 = match digits.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        let minutes = match unit {
            'w' => n.saturating_mul(5 * 8 * 60),
            'd' => n.saturating_mul(8 * 60),
            'h' => n.saturating_mul(60),
            'm' => n,
            _ => return false,
        };
        total_minutes = total_minutes.saturating_add(minutes);
    }
    total_minutes > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_time() {
        assert!(!is_valid_jira_time_format("0h"));
        assert!(!is_valid_jira_time_format("0m"));
        assert!(!is_valid_jira_time_format("0d"));
        assert!(!is_valid_jira_time_format("0w"));
        assert!(!is_valid_jira_time_format("0d 0h 0m"));
        assert!(is_valid_jira_time_format("0h 30m"));
        assert!(is_valid_jira_time_format("1h"));
        assert!(is_valid_jira_time_format("2h 30m"));
    }
}
