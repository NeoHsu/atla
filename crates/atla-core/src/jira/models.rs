use atla_jira_api::types as generated_types;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

use super::util::{adf_plain_text, issue_fields, quote_jql_value};
use crate::client::ApiError;
use crate::markdown::markdown_to_adf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraProjectSearch {
    pub start_at: u64,
    pub max_results: u32,
    pub query: Option<String>,
}

impl Default for JiraProjectSearch {
    fn default() -> Self {
        Self {
            start_at: 0,
            max_results: 50,
            query: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueSearch {
    pub jql: String,
    pub max_results: u32,
    pub fields: Option<Vec<String>>,
}

impl JiraIssueSearch {
    pub(super) fn issue_fields(&self) -> Vec<String> {
        issue_fields(self.fields.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueList {
    pub project_key: Option<String>,
    pub status: Option<String>,
    pub issue_type: Option<String>,
    pub assignee: Option<String>,
    pub jql: Option<String>,
    pub max_results: u32,
    pub fields: Option<Vec<String>>,
}

impl JiraIssueList {
    pub fn to_search(&self, default_project: Option<&str>) -> Result<JiraIssueSearch, ApiError> {
        if let Some(jql) = &self.jql {
            let final_jql = if let Some(project) = self.project_key.as_deref() {
                format!("project = {} AND ({jql})", quote_jql_value(project))
            } else {
                jql.clone()
            };
            return Ok(JiraIssueSearch {
                jql: final_jql,
                max_results: self.max_results,
                fields: self.fields.clone(),
            });
        }

        let project_key = self
            .project_key
            .as_deref()
            .or(default_project)
            .ok_or_else(|| {
                ApiError::Decode(
                    "provide --project or --jql, or set config default-project".to_owned(),
                )
            })?;
        let mut clauses = vec![format!("project = {}", quote_jql_value(project_key))];

        if let Some(status) = &self.status {
            clauses.push(format!("status = {}", quote_jql_value(status)));
        }
        if let Some(issue_type) = &self.issue_type {
            clauses.push(format!("issuetype = {}", quote_jql_value(issue_type)));
        }
        if let Some(assignee) = &self.assignee {
            if assignee.eq_ignore_ascii_case("me") {
                clauses.push("assignee = currentUser()".to_owned());
            } else {
                clauses.push(format!("assignee = {}", quote_jql_value(assignee)));
            }
        }

        Ok(JiraIssueSearch {
            jql: format!("{} ORDER BY updated DESC", clauses.join(" AND ")),
            max_results: self.max_results,
            fields: self.fields.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueAssign {
    pub issue_id_or_key: String,
    pub target: JiraAssigneeTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JiraAssigneeTarget {
    Me,
    AccountId(String),
    Query(String),
    Unassign,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraCreatedIssue {
    pub id: Option<String>,
    pub key: Option<String>,
    pub self_url: Option<String>,
}

impl From<generated_types::CreatedIssue> for JiraCreatedIssue {
    fn from(issue: generated_types::CreatedIssue) -> Self {
        Self {
            id: issue.id,
            key: issue.key,
            self_url: issue.self_,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JiraIssueCreate {
    pub project_key: String,
    pub issue_type: String,
    pub summary: String,
    pub description: Option<String>,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl JiraIssueCreate {
    pub(super) fn to_generated(&self) -> generated_types::IssueUpdateDetails {
        let mut fields = self.fields.clone();
        fields.insert(
            "project".to_owned(),
            serde_json::json!({ "key": self.project_key }),
        );
        fields.insert(
            "issuetype".to_owned(),
            serde_json::json!({ "name": self.issue_type }),
        );
        fields.insert("summary".to_owned(), self.summary.clone().into());
        if let Some(description) = &self.description {
            fields.insert("description".to_owned(), markdown_to_adf(description));
        }

        generated_types::IssueUpdateDetails {
            fields,
            update: serde_json::Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JiraIssueUpdate {
    pub issue_id_or_key: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueLabelUpdate {
    pub issue_id_or_key: String,
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

impl JiraIssueLabelUpdate {
    pub fn is_empty(&self) -> bool {
        self.add.is_empty() && self.remove.is_empty()
    }

    pub(super) fn to_json(&self) -> serde_json::Value {
        let mut operations = Vec::new();
        for label in &self.add {
            operations.push(serde_json::json!({ "add": label }));
        }
        for label in &self.remove {
            operations.push(serde_json::json!({ "remove": label }));
        }

        serde_json::json!({
            "update": {
                "labels": operations
            }
        })
    }
}

impl JiraIssueUpdate {
    pub fn is_empty(&self) -> bool {
        self.summary.is_none() && self.description.is_none() && self.fields.is_empty()
    }

    pub(super) fn to_generated(&self) -> generated_types::IssueUpdateDetails {
        let mut fields = self.fields.clone();
        if let Some(summary) = &self.summary {
            fields.insert("summary".to_owned(), summary.clone().into());
        }
        if let Some(description) = &self.description {
            fields.insert("description".to_owned(), markdown_to_adf(description));
        }

        generated_types::IssueUpdateDetails {
            fields,
            update: serde_json::Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueSearchPage {
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub issues: Vec<JiraIssue>,
}

impl From<generated_types::SearchAndReconcileResults> for JiraIssueSearchPage {
    fn from(page: generated_types::SearchAndReconcileResults) -> Self {
        Self {
            is_last: page.is_last,
            next_page_token: page.next_page_token,
            issues: page.issues.into_iter().map(JiraIssue::from).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssue {
    pub id: Option<String>,
    pub key: Option<String>,
    #[serde(default)]
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl JiraIssue {
    pub fn summary(&self) -> Option<&str> {
        self.field_string("summary")
    }

    pub fn status_name(&self) -> Option<&str> {
        self.field_object_string("status", "name")
    }

    pub fn assignee_display_name(&self) -> Option<&str> {
        self.field_object_string("assignee", "displayName")
    }

    pub fn issue_type_name(&self) -> Option<&str> {
        self.field_object_string("issuetype", "name")
    }

    pub fn priority_name(&self) -> Option<&str> {
        self.field_object_string("priority", "name")
    }

    fn field_string(&self, name: &str) -> Option<&str> {
        self.fields.get(name)?.as_str()
    }

    fn field_object_string(&self, object: &str, name: &str) -> Option<&str> {
        self.fields.get(object)?.get(name)?.as_str()
    }
}

impl From<generated_types::IssueBean> for JiraIssue {
    fn from(issue: generated_types::IssueBean) -> Self {
        Self {
            id: issue.id,
            key: issue.key,
            fields: issue.fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraTransition {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "toStatus", alias = "to")]
    pub to_status: Option<JiraStatus>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl JiraTransition {
    pub fn required_fields(&self) -> Vec<&str> {
        self.fields
            .iter()
            .filter_map(|(id, metadata)| {
                metadata
                    .get("required")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                    .then_some(id.as_str())
            })
            .collect()
    }
}

impl From<generated_types::Transition> for JiraTransition {
    fn from(transition: generated_types::Transition) -> Self {
        Self {
            id: transition.id,
            name: transition.name,
            to_status: transition.to.map(JiraStatus::from),
            fields: serde_json::Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraStatus {
    pub id: Option<String>,
    pub name: Option<String>,
}

impl From<generated_types::Status> for JiraStatus {
    fn from(status: generated_types::Status) -> Self {
        Self {
            id: status.id,
            name: status.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraCommentPage {
    #[serde(default)]
    pub start_at: u32,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u32>,
    #[serde(default)]
    pub comments: Vec<JiraComment>,
}

impl From<generated_types::PageOfComments> for JiraCommentPage {
    fn from(page: generated_types::PageOfComments) -> Self {
        Self {
            start_at: page.start_at.unwrap_or_default().max(0) as u32,
            max_results: page.max_results.unwrap_or_default().max(0) as u32,
            total: page.total.map(|total| total.max(0) as u32),
            comments: page.comments.into_iter().map(JiraComment::from).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComment {
    pub id: Option<String>,
    pub body_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
    pub author_display_name: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

impl From<generated_types::Comment> for JiraComment {
    fn from(comment: generated_types::Comment) -> Self {
        let body_map = if comment.body.is_empty() {
            None
        } else {
            Some(comment.body.clone())
        };
        Self {
            id: comment.id,
            body: body_map
                .as_ref()
                .map(|b| serde_json::Value::Object(b.clone())),
            body_text: body_map
                .as_ref()
                .map(adf_plain_text)
                .filter(|text| !text.is_empty()),
            author_display_name: comment.author.and_then(|author| author.display_name),
            created: comment.created,
            updated: comment.updated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraIssueLinkCreate {
    pub source_key: String,
    pub target_key: String,
    pub link_type: String,
}

impl JiraIssueLinkCreate {
    pub(super) fn to_generated(&self) -> generated_types::LinkIssueRequestJsonBean {
        generated_types::LinkIssueRequestJsonBean {
            type_: Some(generated_types::LinkIssueRequestJsonBeanType {
                name: Some(self.link_type.clone()),
            }),
            inward_issue: Some(generated_types::LinkIssueRequestJsonBeanInwardIssue {
                key: Some(self.source_key.clone()),
            }),
            outward_issue: Some(generated_types::LinkIssueRequestJsonBeanOutwardIssue {
                key: Some(self.target_key.clone()),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueLink {
    pub id: Option<String>,
    pub link_type: Option<String>,
    pub inward_issue: Option<JiraLinkedIssue>,
    pub outward_issue: Option<JiraLinkedIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraAttachment {
    #[serde(default, deserialize_with = "deserialize_string_or_number")]
    pub id: Option<String>,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
    pub author: Option<JiraUser>,
    pub created: Option<String>,
    pub content: Option<String>,
    pub thumbnail: Option<String>,
}

impl From<generated_types::Attachment> for JiraAttachment {
    fn from(attachment: generated_types::Attachment) -> Self {
        Self {
            id: attachment.id,
            filename: attachment.filename,
            mime_type: attachment.mime_type,
            size: attachment.size.map(|s| s as u64),
            author: attachment.author.map(|u| JiraUser {
                account_id: u.account_id,
                display_name: u.display_name,
                active: u.active,
            }),
            created: attachment.created,
            content: attachment.content,
            thumbnail: attachment.thumbnail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraAttachmentDownload {
    pub attachment: JiraAttachment,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraLinkedIssue {
    pub id: Option<String>,
    pub key: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraUser {
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraBoardSearch {
    pub start_at: u64,
    pub max_results: u32,
    pub board_type: Option<String>,
    pub name: Option<String>,
    pub project_key_or_id: Option<String>,
}

impl Default for JiraBoardSearch {
    fn default() -> Self {
        Self {
            start_at: 0,
            max_results: 50,
            board_type: None,
            name: None,
            project_key_or_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraBoardPage {
    #[serde(default)]
    pub start_at: u64,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub values: Vec<JiraBoard>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraBoard {
    pub id: Option<u64>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub board_type: Option<String>,
    #[serde(rename = "self")]
    pub self_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraSprintSearch {
    pub board_id: u64,
    pub start_at: u64,
    pub max_results: u32,
    pub state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSprintPage {
    #[serde(default)]
    pub start_at: u64,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub values: Vec<JiraSprint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSprint {
    pub id: Option<u64>,
    #[serde(rename = "self")]
    pub self_url: Option<String>,
    pub state: Option<String>,
    pub name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub complete_date: Option<String>,
    pub origin_board_id: Option<u64>,
    pub goal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraSprintCreate {
    pub board_id: u64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub goal: Option<String>,
}

impl JiraSprintCreate {
    pub(super) fn to_json(&self) -> serde_json::Value {
        let mut value = serde_json::json!({
            "originBoardId": self.board_id,
            "name": self.name,
        });
        let object = value.as_object_mut().expect("sprint create is object");
        if let Some(start_date) = &self.start_date {
            object.insert("startDate".to_owned(), start_date.clone().into());
        }
        if let Some(end_date) = &self.end_date {
            object.insert("endDate".to_owned(), end_date.clone().into());
        }
        if let Some(goal) = &self.goal {
            object.insert("goal".to_owned(), goal.clone().into());
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraSprintUpdate {
    pub id: u64,
    pub state: Option<String>,
    pub name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub goal: Option<String>,
}

impl JiraSprintUpdate {
    pub(super) fn to_json(&self) -> serde_json::Value {
        let mut object = serde_json::Map::new();
        if let Some(state) = &self.state {
            object.insert("state".to_owned(), state.clone().into());
        }
        if let Some(name) = &self.name {
            object.insert("name".to_owned(), name.clone().into());
        }
        if let Some(start_date) = &self.start_date {
            object.insert("startDate".to_owned(), start_date.clone().into());
        }
        if let Some(end_date) = &self.end_date {
            object.insert("endDate".to_owned(), end_date.clone().into());
        }
        if let Some(goal) = &self.goal {
            object.insert("goal".to_owned(), goal.clone().into());
        }
        serde_json::Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraWorklogCreate {
    pub issue_id_or_key: String,
    pub time_spent: String,
    pub comment: Option<String>,
    pub started: Option<String>,
}

impl JiraWorklogCreate {
    pub(super) fn to_json(&self) -> serde_json::Value {
        let mut value = serde_json::json!({ "timeSpent": self.time_spent });
        let obj = value.as_object_mut().expect("worklog create is object");
        if let Some(comment) = &self.comment {
            obj.insert("comment".to_owned(), markdown_to_adf(comment));
        }
        if let Some(started) = &self.started {
            obj.insert(
                "started".to_owned(),
                serde_json::Value::String(started.clone()),
            );
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraWorklogPage {
    #[serde(default)]
    pub start_at: u32,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u32>,
    #[serde(default)]
    pub worklogs: Vec<JiraWorklog>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraWorklog {
    pub id: Option<String>,
    pub time_spent: Option<String>,
    pub time_spent_seconds: Option<u64>,
    pub author: Option<JiraWorklogAuthor>,
    pub comment: Option<serde_json::Value>,
    pub started: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraWorklogAuthor {
    pub display_name: Option<String>,
    pub account_id: Option<String>,
}

impl JiraWorklog {
    pub fn comment_text(&self) -> Option<String> {
        let object = self.comment.as_ref()?.as_object()?;
        Some(adf_plain_text(&object.clone().into_iter().collect())).filter(|text| !text.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueType {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub subtask: Option<bool>,
}

impl From<generated_types::IssueType> for JiraIssueType {
    fn from(issue_type: generated_types::IssueType) -> Self {
        Self {
            id: issue_type.id,
            name: issue_type.name,
            description: issue_type.description,
            subtask: issue_type.subtask,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraProjectPage {
    #[serde(default)]
    pub start_at: u64,
    #[serde(default)]
    pub max_results: u32,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub values: Vec<JiraProject>,
}

impl From<generated_types::PageBeanProject> for JiraProjectPage {
    fn from(page: generated_types::PageBeanProject) -> Self {
        Self {
            start_at: page.start_at.unwrap_or_default().max(0) as u64,
            max_results: page.max_results.unwrap_or_default().max(0) as u32,
            total: page.total.map(|total| total.max(0) as u64),
            is_last: page.is_last,
            values: page.values.into_iter().map(JiraProject::from).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraProject {
    pub id: Option<String>,
    pub key: Option<String>,
    pub name: Option<String>,
    pub project_type_key: Option<String>,
    pub style: Option<String>,
    pub simplified: Option<bool>,
    pub archived: Option<bool>,
}

impl From<generated_types::Project> for JiraProject {
    fn from(project: generated_types::Project) -> Self {
        Self {
            id: project.id,
            key: project.key,
            name: project.name,
            project_type_key: project.project_type_key.map(|k| k.to_string()),
            style: project.style.map(|s| s.to_string()),
            simplified: project.simplified,
            archived: project.archived,
        }
    }
}

/// Deserializes a value that may be either a JSON string or number into `Option<String>`.
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    Ok(value.and_then(|v| match v {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Number(n) => Some(n.to_string()),
        _ => None,
    }))
}
