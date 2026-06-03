use atla_jira_api::types as generated_types;

use super::JiraClient;
use super::models::{
    JiraAssigneeTarget, JiraCreatedIssue, JiraIssue, JiraIssueAssign, JiraIssueCreate,
    JiraIssueField, JiraIssueFieldsQuery, JiraIssueLabelUpdate, JiraIssueSearch,
    JiraIssueSearchPage, JiraIssueUpdate, JiraTransition, JiraUser,
};
use super::util::{generated_error, generated_error_with_body, issue_fields, limit_i32};
use crate::client::{ApiError, read_empty, read_json};

/// Per-request maximum for `GET /rest/api/3/search/jql`. The server silently
/// caps `maxResults` to this value, so larger callers must paginate to reach it.
const JIRA_JQL_SEARCH_PAGE_CAP: u32 = 100;

impl JiraClient {
    pub async fn create_issue(
        &self,
        issue: &JiraIssueCreate,
    ) -> Result<JiraCreatedIssue, ApiError> {
        match self
            .generated
            .create_issue()
            .body(issue.to_generated())
            .send()
            .await
        {
            Ok(rv) => Ok(JiraCreatedIssue::from(rv.into_inner())),
            Err(e) => Err(generated_error_with_body(e).await),
        }
    }

    pub async fn update_issue(&self, issue: &JiraIssueUpdate) -> Result<(), ApiError> {
        match self
            .generated
            .edit_issue()
            .issue_id_or_key(&issue.issue_id_or_key)
            .body(issue.to_generated())
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(generated_error_with_body(e).await),
        }
    }

    pub async fn update_issue_labels(&self, labels: &JiraIssueLabelUpdate) -> Result<(), ApiError> {
        let json = labels.to_json();
        let update_map = json
            .get("update")
            .and_then(serde_json::Value::as_object)
            .cloned()
            .unwrap_or_default();

        let details = generated_types::IssueUpdateDetails {
            fields: serde_json::Map::new(),
            update: update_map,
        };

        match self
            .generated
            .edit_issue()
            .issue_id_or_key(&labels.issue_id_or_key)
            .body(details)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(generated_error_with_body(e).await),
        }
    }

    pub async fn search_issues(
        &self,
        search: &JiraIssueSearch,
    ) -> Result<JiraIssueSearchPage, ApiError> {
        let fields = search.issue_fields();
        let mut collected: Vec<JiraIssue> = Vec::new();
        let mut next_page_token: Option<String> = search.next_page_token.clone();
        let mut server_is_last: Option<bool> = Some(true);

        while (collected.len() as u32) < search.max_results {
            let remaining = search.max_results - collected.len() as u32;
            let page_size = remaining.min(JIRA_JQL_SEARCH_PAGE_CAP);

            let mut builder = self
                .generated
                .search_and_reconsile_issues_using_jql()
                .jql(&search.jql)
                .max_results(limit_i32(page_size))
                .fields(fields.clone());
            if let Some(token) = &next_page_token {
                builder = builder.next_page_token(token.clone());
            }

            let mut page = JiraIssueSearchPage::from(
                builder.send().await.map_err(generated_error)?.into_inner(),
            );

            let received = page.issues.len();
            server_is_last = page.is_last;
            next_page_token = page.next_page_token.take();
            collected.append(&mut page.issues);

            if received == 0 || matches!(server_is_last, Some(true)) || next_page_token.is_none() {
                break;
            }
        }

        if (collected.len() as u32) > search.max_results {
            collected.truncate(search.max_results as usize);
        }

        let exhausted = matches!(server_is_last, Some(true)) || next_page_token.is_none();
        Ok(JiraIssueSearchPage {
            is_last: Some(exhausted),
            next_page_token: if exhausted { None } else { next_page_token },
            issues: collected,
        })
    }

    pub async fn get_issue(
        &self,
        issue_id_or_key: &str,
        fields: Option<Vec<String>>,
    ) -> Result<JiraIssue, ApiError> {
        let builder = self
            .generated
            .get_issue()
            .issue_id_or_key(issue_id_or_key)
            .fields(issue_fields(fields.as_deref()));

        builder
            .send()
            .await
            .map(|rv| JiraIssue::from(rv.into_inner()))
            .map_err(generated_error)
    }

    pub async fn list_transitions(
        &self,
        issue_id_or_key: &str,
    ) -> Result<Vec<JiraTransition>, ApiError> {
        let transitions = self
            .generated
            .get_transitions()
            .issue_id_or_key(issue_id_or_key)
            .expand("transitions.fields")
            .send()
            .await
            .map_err(generated_error)?;

        Ok(transitions
            .into_inner()
            .transitions
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

        let transition_request = generated_types::IssueTransitionRequest {
            transition: generated_types::IssueTransitionRequestTransition { id: transition_id },
            fields: if fields.is_empty() {
                serde_json::Map::new()
            } else {
                fields
            },
        };
        match self
            .generated
            .do_transition()
            .issue_id_or_key(issue_id_or_key)
            .body(transition_request)
            .send()
            .await
        {
            Ok(_) => {}
            Err(e) => return Err(generated_error_with_body(e).await),
        }

        Ok(transition)
    }

    pub async fn delete_issue(
        &self,
        issue_id_or_key: &str,
        delete_subtasks: bool,
    ) -> Result<(), ApiError> {
        self.generated
            .delete_issue()
            .issue_id_or_key(issue_id_or_key)
            .delete_subtasks(delete_subtasks)
            .send()
            .await
            .map(|_| ())
            .map_err(generated_error)
    }

    pub async fn assign_issue(&self, assign: &JiraIssueAssign) -> Result<JiraUser, ApiError> {
        if matches!(&assign.target, JiraAssigneeTarget::Unassign) {
            read_empty(
                self.raw_client
                    .put(&format!(
                        "/rest/api/3/issue/{}/assignee",
                        assign.issue_id_or_key
                    ))
                    .json(&serde_json::json!({ "accountId": null })),
            )
            .await?;

            return Ok(JiraUser {
                account_id: None,
                display_name: None,
                active: None,
            });
        }

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
            JiraAssigneeTarget::Unassign => unreachable!(),
        };
        let account_id = user.account_id.clone().ok_or_else(|| {
            ApiError::Decode("selected Jira user did not include an accountId".to_owned())
        })?;

        read_empty(
            self.raw_client
                .put(&format!(
                    "/rest/api/3/issue/{}/assignee",
                    assign.issue_id_or_key
                ))
                .json(&serde_json::json!({ "accountId": account_id })),
        )
        .await?;

        Ok(user)
    }

    async fn current_user(&self) -> Result<JiraUser, ApiError> {
        read_json(self.raw_client.get("/rest/api/3/myself")).await
    }

    pub async fn get_issue_fields(
        &self,
        query: &JiraIssueFieldsQuery,
    ) -> Result<Vec<JiraIssueField>, ApiError> {
        let page_size = 50_i32;
        let mut all_fields: Vec<JiraIssueField> = Vec::new();
        let mut start_at = 0_i32;

        loop {
            let page = match self
                .generated
                .get_create_issue_meta_issue_type_id()
                .project_id_or_key(&query.project_key)
                .issue_type_id(&query.issue_type_id)
                .max_results(page_size)
                .start_at(start_at)
                .send()
                .await
            {
                Ok(rv) => rv.into_inner(),
                Err(e) => return Err(generated_error_with_body(e).await),
            };

            let total = page.total.unwrap_or(i32::MAX);
            let fetched = page.fields.len() as i32;
            all_fields.extend(JiraIssueField::from_page(page));

            if fetched == 0 {
                break;
            }
            start_at += fetched;
            if start_at >= total {
                break;
            }
        }

        Ok(all_fields)
    }

    async fn find_assignable_users(
        &self,
        issue_id_or_key: &str,
        query_text: &str,
    ) -> Result<Vec<JiraUser>, ApiError> {
        read_json(
            self.raw_client
                .get("/rest/api/3/user/assignable/search")
                .query(&[
                    ("issueKey", issue_id_or_key),
                    ("query", query_text),
                    ("maxResults", "50"),
                ]),
        )
        .await
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
