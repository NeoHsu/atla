use atla_jira_api::apis as generated_apis;

use super::JiraClient;
use super::models::{JiraIssueType, JiraProject, JiraProjectPage, JiraProjectSearch};
use super::util::{generated_error, limit_i32};
use crate::client::ApiError;

impl JiraClient {
    pub async fn search_projects(
        &self,
        search: &JiraProjectSearch,
    ) -> Result<JiraProjectPage, ApiError> {
        let page = generated_apis::projects_api::search_projects(
            &self.generated,
            Some(search.start_at.min(i64::MAX as u64) as i64),
            Some(limit_i32(search.max_results)),
            search.query.as_deref(),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn get_project(&self, project_id_or_key: &str) -> Result<JiraProject, ApiError> {
        generated_apis::projects_api::get_project(&self.generated, project_id_or_key)
            .await
            .map(JiraProject::from)
            .map_err(generated_error)
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

        let issue_types = generated_apis::issue_types_api::get_issue_types_for_project(
            &self.generated,
            Some(&project_id),
        )
        .await
        .map_err(generated_error)?;

        Ok(issue_types.into_iter().map(JiraIssueType::from).collect())
    }
}
