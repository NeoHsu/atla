use super::JiraClient;
use super::models::{JiraIssueType, JiraProject, JiraProjectPage, JiraProjectSearch};
use super::util::{generated_error, generated_error_with_body, limit_i32};
use crate::client::ApiError;

impl JiraClient {
    pub async fn search_projects(
        &self,
        search: &JiraProjectSearch,
    ) -> Result<JiraProjectPage, ApiError> {
        let mut builder = self
            .generated
            .search_projects()
            .start_at(search.start_at.min(i64::MAX as u64) as i64)
            .max_results(limit_i32(search.max_results));

        if let Some(query) = &search.query {
            builder = builder.query(query.clone());
        }

        let page = builder.send().await.map_err(generated_error)?;
        Ok(page.into_inner().into())
    }

    pub async fn get_project(&self, project_id_or_key: &str) -> Result<JiraProject, ApiError> {
        match self
            .generated
            .get_project()
            .project_id_or_key(project_id_or_key)
            .send()
            .await
        {
            Ok(rv) => Ok(JiraProject::from(rv.into_inner())),
            Err(e) => Err(generated_error_with_body(e).await),
        }
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

        let issue_types = self
            .generated
            .get_issue_types_for_project()
            .project_id(&project_id)
            .send()
            .await
            .map_err(generated_error)?;

        let list: Vec<_> = Vec::from(issue_types.into_inner());
        Ok(list.into_iter().map(JiraIssueType::from).collect())
    }
}
