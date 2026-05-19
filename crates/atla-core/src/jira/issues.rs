use atla_jira_api::{apis as generated_apis, models as generated_models};

use super::JiraClient;
use super::models::{
    JiraAssigneeTarget, JiraCreatedIssue, JiraIssue, JiraIssueAssign, JiraIssueCreate,
    JiraIssueLabelUpdate, JiraIssueSearch, JiraIssueSearchPage, JiraIssueUpdate, JiraTransition,
    JiraUser,
};
use super::util::{generated_error, issue_fields, limit_i32};
use crate::client::ApiError;

impl JiraClient {
    pub async fn create_issue(
        &self,
        issue: &JiraIssueCreate,
    ) -> Result<JiraCreatedIssue, ApiError> {
        generated_apis::issues_api::create_issue(&self.generated, issue.to_generated())
            .await
            .map(JiraCreatedIssue::from)
            .map_err(generated_error)
    }

    pub async fn update_issue(&self, issue: &JiraIssueUpdate) -> Result<(), ApiError> {
        generated_apis::issues_api::edit_issue(
            &self.generated,
            &issue.issue_id_or_key,
            issue.to_generated(),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn update_issue_labels(&self, labels: &JiraIssueLabelUpdate) -> Result<(), ApiError> {
        let json = labels.to_json();
        let update_map = json
            .get("update")
            .and_then(serde_json::Value::as_object)
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect());

        let details = generated_models::IssueUpdateDetails {
            fields: None,
            update: update_map,
        };

        generated_apis::issues_api::edit_issue(&self.generated, &labels.issue_id_or_key, details)
            .await
            .map_err(generated_error)
    }

    pub async fn search_issues(
        &self,
        search: &JiraIssueSearch,
    ) -> Result<JiraIssueSearchPage, ApiError> {
        let page = generated_apis::issue_search_api::search_and_reconsile_issues_using_jql(
            &self.generated,
            Some(&search.jql),
            None,
            Some(limit_i32(search.max_results)),
            Some(search.issue_fields()),
        )
        .await
        .map_err(generated_error)?;

        Ok(page.into())
    }

    pub async fn get_issue(
        &self,
        issue_id_or_key: &str,
        fields: Option<Vec<String>>,
    ) -> Result<JiraIssue, ApiError> {
        generated_apis::issues_api::get_issue(
            &self.generated,
            issue_id_or_key,
            Some(issue_fields(fields.as_deref())),
        )
        .await
        .map(JiraIssue::from)
        .map_err(generated_error)
    }

    pub async fn list_transitions(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraTransition>, ApiError> {
        let transitions = generated_apis::issues_api::get_transitions(
            &self.generated,
            issue_id_or_key,
            Some("transitions.fields"),
        )
        .await
        .map_err(generated_error)?;

        Ok(transitions
            .transitions
            .unwrap_or_default()
            .into_iter()
            .map(JiraTransition::from)
            .collect())
    }

    pub async fn transition_issue(
        &self,
        issue_id_or_key: &str,
        transition_id_or_name: &str,
        fields: serde_json::Map<String, serde_json::Value>,
    ) -> Result<JiraTransition, ApiError> {
        let transitions = self.list_transitions(issue_id_or_key).await?;
        let transition = transitions
            .iter()
            .find(|transition| {
                transition.id.as_deref() == Some(transition_id_or_name)
                    || transition
                        .name
                        .as_deref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(transition_id_or_name))
            })
            .cloned()
            .ok_or_else(|| {
                let available = transitions
                    .iter()
                    .filter_map(|transition| transition.name.as_deref())
                    .collect::<Vec<_>>()
                    .join(", ");
                if available.is_empty() {
                    ApiError::Decode(format!(
                        "transition `{transition_id_or_name}` not available for issue `{issue_id_or_key}`"
                    ))
                } else {
                    ApiError::Decode(format!(
                        "transition `{transition_id_or_name}` not available for issue `{issue_id_or_key}`; available: {available}"
                    ))
                }
            })?;
        let transition_id = transition.id.clone().ok_or_else(|| {
            ApiError::Decode("selected transition did not include an id".to_owned())
        })?;
        let missing_fields = transition
            .required_fields()
            .into_iter()
            .filter(|field| !fields.contains_key(*field))
            .collect::<Vec<_>>();
        if !missing_fields.is_empty() {
            return Err(ApiError::Decode(format!(
                "transition `{transition_id_or_name}` requires field(s): {}",
                missing_fields.join(", ")
            )));
        }

        let transition_request = generated_models::IssueTransitionRequest {
            transition: Box::new(generated_models::IssueTransitionRequestTransition::new(
                transition_id,
            )),
            fields: if fields.is_empty() {
                None
            } else {
                Some(fields.into_iter().collect())
            },
        };
        generated_apis::issues_api::do_transition(
            &self.generated,
            issue_id_or_key,
            transition_request,
        )
        .await
        .map_err(generated_error)?;

        Ok(transition)
    }

    pub async fn delete_issue(
        &self,
        issue_id_or_key: &str,
        delete_subtasks: bool,
    ) -> Result<(), ApiError> {
        generated_apis::issues_api::delete_issue(
            &self.generated,
            issue_id_or_key,
            Some(delete_subtasks),
        )
        .await
        .map_err(generated_error)
    }

    pub async fn assign_issue(&self, assign: &JiraIssueAssign) -> Result<JiraUser, ApiError> {
        let user = match &assign.target {
            JiraAssigneeTarget::Me => self.current_user().await?,
            JiraAssigneeTarget::AccountId(account_id) => JiraUser {
                account_id: Some(account_id.clone()),
                display_name: None,
                active: None,
            },
            JiraAssigneeTarget::Query(query) => {
                let users = self
                    .find_assignable_users(&assign.issue_id_or_key, query)
                    .await?;
                resolve_assignable_user(query, users)?
            }
        };
        let account_id = user.account_id.clone().ok_or_else(|| {
            ApiError::Decode("selected Jira user did not include an accountId".to_owned())
        })?;

        generated_apis::issues_api::set_assignee(
            &self.generated,
            &assign.issue_id_or_key,
            Some(account_id),
        )
        .await
        .map_err(generated_error)?;

        Ok(user)
    }

    async fn current_user(&self) -> Result<JiraUser, ApiError> {
        let value = generated_apis::users_api::get_myself(&self.generated)
            .await
            .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    async fn find_assignable_users(
        &self,
        issue_id_or_key: &str,
        query_text: &str,
    ) -> Result<Vec<JiraUser>, ApiError> {
        let value = generated_apis::users_api::find_assignable_users_for_issue(
            &self.generated,
            issue_id_or_key,
            query_text,
            Some(50),
        )
        .await
        .map_err(generated_error)?;

        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }
}

fn resolve_assignable_user(query: &str, users: Vec<JiraUser>) -> Result<JiraUser, ApiError> {
    let exact_matches = users
        .iter()
        .filter(|user| {
            user.account_id.as_deref() == Some(query)
                || user
                    .display_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(query))
        })
        .cloned()
        .collect::<Vec<_>>();
    if exact_matches.len() == 1 {
        return Ok(exact_matches.into_iter().next().expect("one exact match"));
    }

    match users.as_slice() {
        [user] => Ok(user.clone()),
        [] => Err(ApiError::Decode(format!(
            "no assignable Jira user matched `{query}`"
        ))),
        _ => {
            let names = users
                .iter()
                .filter_map(|user| user.display_name.as_deref())
                .collect::<Vec<_>>()
                .join(", ");
            Err(ApiError::Decode(format!(
                "multiple assignable Jira users matched `{query}`; pass --account-id. matches: {names}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_assignable_user_by_exact_display_name() {
        let user = resolve_assignable_user(
            "Neo",
            vec![
                JiraUser {
                    account_id: Some("account-1".to_owned()),
                    display_name: Some("Neo".to_owned()),
                    active: Some(true),
                },
                JiraUser {
                    account_id: Some("account-2".to_owned()),
                    display_name: Some("Neon".to_owned()),
                    active: Some(true),
                },
            ],
        )
        .expect("resolved user");

        assert_eq!(user.account_id.as_deref(), Some("account-1"));
    }

    #[test]
    fn rejects_ambiguous_assignable_users() {
        let error = resolve_assignable_user(
            "neo",
            vec![
                JiraUser {
                    account_id: Some("account-1".to_owned()),
                    display_name: Some("Neo One".to_owned()),
                    active: Some(true),
                },
                JiraUser {
                    account_id: Some("account-2".to_owned()),
                    display_name: Some("Neo Two".to_owned()),
                    active: Some(true),
                },
            ],
        )
        .expect_err("ambiguous user");

        assert!(error.to_string().contains("multiple assignable Jira users"));
    }

    #[test]
    fn resolve_single_user_no_exact_match() {
        let user = resolve_assignable_user(
            "anything",
            vec![JiraUser {
                account_id: Some("account-x".to_owned()),
                display_name: Some("Xavier".to_owned()),
                active: Some(true),
            }],
        )
        .expect("single user");

        assert_eq!(user.account_id.as_deref(), Some("account-x"));
    }

    #[test]
    fn resolve_user_by_account_id() {
        let user = resolve_assignable_user(
            "account-2",
            vec![
                JiraUser {
                    account_id: Some("account-1".to_owned()),
                    display_name: Some("Alice".to_owned()),
                    active: Some(true),
                },
                JiraUser {
                    account_id: Some("account-2".to_owned()),
                    display_name: Some("Bob".to_owned()),
                    active: Some(true),
                },
            ],
        )
        .expect("resolved by account id");

        assert_eq!(user.display_name.as_deref(), Some("Bob"));
    }

    #[test]
    fn resolve_no_users_fails() {
        let error = resolve_assignable_user("ghost", vec![]).expect_err("no users should fail");

        assert!(
            error
                .to_string()
                .contains("no assignable Jira user matched")
        );
    }
}
