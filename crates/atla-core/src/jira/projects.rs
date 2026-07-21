use super::JiraClient;
use super::models::{JiraIssueType, JiraProject, JiraProjectPage, JiraProjectSearch};
use super::util::{JIRA_LIST_PAGE_CAP, generated_request, limit_i32, next_offset};
use crate::client::ApiError;

impl JiraClient {
    pub async fn search_projects(
        &self,
        search: &JiraProjectSearch,
    ) -> Result<JiraProjectPage, ApiError> {
        let max_results = self.raw_client.effective_item_limit(search.max_results);
        let mut collected: Vec<JiraProject> = Vec::new();
        let mut start_at = search.start_at;
        let mut last_is_last: Option<bool> = Some(true);
        let mut last_total: Option<u64> = None;

        while self.raw_client.take_page() {
            let remaining = (max_results as u64).saturating_sub(collected.len() as u64);
            if remaining == 0 {
                break;
            }
            let page_size = remaining.min(JIRA_LIST_PAGE_CAP as u64) as u32;

            let page: JiraProjectPage = generated_request(reqwest::Method::GET, || {
                let mut request = self
                    .generated
                    .search_projects()
                    .start_at(start_at.min(i64::MAX as u64) as i64)
                    .max_results(limit_i32(page_size));
                if let Some(query) = &search.query {
                    request = request.query(query.clone());
                }
                request.send()
            })
            .await?
            .into_inner()
            .into();
            let received = page.values.len() as u64;
            last_is_last = page.is_last;
            last_total = page.total;
            collected.extend(page.values);

            match next_offset(
                collected.len() as u64,
                max_results as u64,
                received,
                last_is_last,
                last_total,
                start_at,
            ) {
                Some(next) => start_at = next,
                None => break,
            }
        }

        let exhausted = matches!(last_is_last, Some(true))
            || last_total.is_some_and(|total| collected.len() as u64 >= total);
        if collected.len() > max_results as usize {
            collected.truncate(max_results as usize);
        }

        Ok(JiraProjectPage {
            start_at: search.start_at,
            max_results,
            total: last_total,
            is_last: Some(exhausted),
            values: collected,
        })
    }

    pub async fn get_project(&self, project_id_or_key: &str) -> Result<JiraProject, ApiError> {
        generated_request(reqwest::Method::GET, || {
            self.generated
                .get_project()
                .project_id_or_key(project_id_or_key)
                .send()
        })
        .await
        .map(|response| JiraProject::from(response.into_inner()))
    }

    pub async fn list_issue_types(
        &self,
        project_id_or_key: &str,
    ) -> Result<Vec<JiraIssueType>, ApiError> {
        let project = self.get_project(project_id_or_key).await?;
        let project_id = project.id.ok_or_else(|| {
            ApiError::Decode(format!(
                "project `{project_id_or_key}` did not include an id"
            ))
        })?;

        let issue_types = generated_request(reqwest::Method::GET, || {
            self.generated
                .get_issue_types_for_project()
                .project_id(&project_id)
                .send()
        })
        .await?;

        let list: Vec<_> = Vec::from(issue_types.into_inner());
        Ok(list.into_iter().map(JiraIssueType::from).collect())
    }
}
