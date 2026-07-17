use super::JiraClient;
use super::models::{JiraWorklog, JiraWorklogCreate, JiraWorklogPage};
use super::util::{JIRA_LIST_PAGE_CAP, limit_i32, next_offset};
use crate::client::{ApiError, read_json};

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

        read_json(
            self.raw_client
                .post(&format!(
                    "/rest/api/3/issue/{}/worklog",
                    worklog.issue_id_or_key
                ))
                .json(&json),
        )
        .await
    }

    pub async fn list_worklogs(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
    ) -> Result<JiraWorklogPage, ApiError> {
        self.list_worklogs_from(issue_id_or_key, max_results, 0)
            .await
    }

    pub async fn list_worklogs_from(
        &self,
        issue_id_or_key: &str,
        max_results: u32,
        start_at: u64,
    ) -> Result<JiraWorklogPage, ApiError> {
        let max_results = self.raw_client.effective_item_limit(max_results);
        let mut collected: Vec<JiraWorklog> = Vec::new();
        let initial_start_at = start_at;
        let mut start_at: u64 = start_at;
        let mut last_total: Option<u32> = None;

        while self.raw_client.take_page() {
            let remaining = (max_results as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(JIRA_LIST_PAGE_CAP as u64) as u32;

            let page: JiraWorklogPage = read_json(
                self.raw_client
                    .get(&format!("/rest/api/3/issue/{issue_id_or_key}/worklog"))
                    .query(&[
                        ("startAt", &start_at.min(i64::MAX as u64).to_string()),
                        ("maxResults", &limit_i32(page_size).to_string()),
                    ]),
            )
            .await?;

            let received = page.worklogs.len() as u64;
            last_total = page.total;
            collected.extend(page.worklogs);

            match next_offset(
                collected.len() as u64,
                max_results as u64,
                received,
                None,
                last_total.map(u64::from),
                start_at,
            ) {
                Some(next) => start_at = next,
                None => break,
            }
        }

        if collected.len() > max_results as usize {
            collected.truncate(max_results as usize);
        }

        Ok(JiraWorklogPage {
            start_at: initial_start_at.min(u32::MAX as u64) as u32,
            max_results,
            total: last_total,
            worklogs: collected,
        })
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
