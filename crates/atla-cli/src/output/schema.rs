use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;
pub const PLAN_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub is_last: bool,
    pub next_page_token: Option<String>,
    pub next_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub schema_version: u32,
    pub error: ErrorBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub kind: &'static str,
    pub message: String,
    pub status: Option<u16>,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationPlan {
    pub schema_version: u32,
    pub plan_version: u32,
    pub operation: String,
    pub profile: String,
    pub site: String,
    pub requests: Vec<PlannedRequest>,
    pub preconditions: Vec<String>,
    pub unresolved: Vec<String>,
    #[serde(default)]
    pub input_files: Vec<InputFileDigest>,
    pub mutating: bool,
    pub created_at: String,
    pub expires_at: String,
    pub plan_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputFileDigest {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlannedRequest {
    pub method: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationReceipt<T> {
    pub schema_version: u32,
    pub operation: &'static str,
    pub profile: String,
    pub target: Option<String>,
    pub request_id: Option<String>,
    #[serde(flatten)]
    pub result: T,
    pub completed_at: String,
}
