use std::io::Read;
use std::path::Path;

use anyhow::Context;
use atla_core::client::read_json;
use atla_core::{AtlassianProduct, Profile};
use chrono::{DateTime, Duration, Utc};
use reqwest::{Method, Url};
use sha2::{Digest, Sha256};

use crate::cli::GlobalArgs;
use crate::context::AppContext;
use crate::error::UsageError;
use crate::output;
use crate::output::schema::{OperationPlan, PlannedRequest};

pub async fn apply(path: &Path, yes: bool, global: &GlobalArgs) -> anyhow::Result<()> {
    if !yes {
        return usage("refusing to apply a mutation plan without --yes");
    }
    let plan = load_and_verify(path)?;
    let ctx = AppContext::load(global)?;
    if plan.profile != ctx.profile_name() {
        return usage(format!(
            "plan targets profile `{}`, but active profile is `{}`",
            plan.profile,
            ctx.profile_name()
        ));
    }
    if plan.site.trim_end_matches('/') != ctx.profile().instance.trim_end_matches('/') {
        return usage(format!(
            "plan site `{}` does not match profile site `{}`",
            plan.site,
            ctx.profile().instance
        ));
    }
    if !ctx.profile().policy.allows(&plan.operation, true) {
        return usage(format!(
            "operation `{}` is blocked by policy for profile `{}`",
            plan.operation,
            ctx.profile_name()
        ));
    }
    if !plan.unresolved.is_empty() {
        return usage(format!(
            "plan has unresolved values: {}",
            plan.unresolved.join(", ")
        ));
    }
    verify_input_files(&plan)?;
    let request = plan
        .requests
        .first()
        .ok_or_else(|| anyhow::Error::new(UsageError("plan has no request".to_owned())))?;
    if plan.requests.len() != 1 {
        return usage("only single-request plans are supported");
    }
    let product = validate_request(&plan.operation, request, ctx.profile())?;
    let body = request.body.as_ref().ok_or_else(|| {
        anyhow::Error::new(UsageError("plan request has no JSON body".to_owned()))
    })?;
    if !body.is_object() {
        return usage("plan request body must be a JSON object");
    }

    output::configure_operation(plan.operation.clone(), true, false);
    output::configure_profile(ctx.profile_name());
    let client = ctx.atlassian_client(product)?;
    let builder = match request.method.as_str() {
        "POST" => client.post(&request.url),
        "PUT" => client.put(&request.url),
        _ => return usage("plan request method is not allowed"),
    }
    .json(body);
    let result: serde_json::Value = read_json(builder)
        .await
        .with_context(|| format!("failed to apply operation `{}`", plan.operation))?;
    if result.is_object() {
        output::print_json(&result)
    } else {
        output::print_json(&serde_json::json!({ "result": result }))
    }
}

fn load_and_verify(path: &Path) -> anyhow::Result<OperationPlan> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("failed to inspect plan `{}`", path.display()))?;
    if metadata.len() > output::MAX_PLAN_BYTES {
        return usage(format!(
            "plan is {} bytes; maximum is {}",
            metadata.len(),
            output::MAX_PLAN_BYTES
        ));
    }
    let bytes =
        std::fs::read(path).with_context(|| format!("failed to read plan `{}`", path.display()))?;
    let plan: OperationPlan = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse plan `{}`", path.display()))?;
    if plan.schema_version != output::schema::SCHEMA_VERSION {
        return usage(format!(
            "unsupported schemaVersion {}; expected {}",
            plan.schema_version,
            output::schema::SCHEMA_VERSION
        ));
    }
    if plan.plan_version != output::schema::PLAN_VERSION {
        return usage(format!(
            "unsupported planVersion {}; expected {}",
            plan.plan_version,
            output::schema::PLAN_VERSION
        ));
    }
    if !plan.mutating {
        return usage("apply accepts only mutating plans");
    }
    if !crate::operation::supports_saved_plan(&plan.operation) {
        return usage(format!(
            "operation `{}` is not allowed in saved plans",
            plan.operation
        ));
    }
    let expected_hash = output::operation_plan_hash(&plan)?;
    if plan.plan_hash != expected_hash {
        return usage("plan hash mismatch; the plan was modified or corrupted");
    }
    let created_at = parse_time("createdAt", &plan.created_at)?;
    let expires_at = parse_time("expiresAt", &plan.expires_at)?;
    let now = Utc::now();
    if created_at > now + Duration::minutes(5) {
        return usage("plan createdAt is too far in the future");
    }
    if expires_at <= created_at || expires_at - created_at > Duration::hours(24) {
        return usage("plan validity window must be greater than zero and at most 24 hours");
    }
    if expires_at <= now {
        return usage("plan has expired; create a new plan");
    }
    Ok(plan)
}

fn parse_time(field: &str, value: &str) -> anyhow::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            anyhow::Error::new(UsageError(format!("plan {field} is not RFC 3339: {error}")))
        })
}

fn verify_input_files(plan: &OperationPlan) -> anyhow::Result<()> {
    for input in &plan.input_files {
        let mut file = std::fs::File::open(&input.path)
            .with_context(|| format!("failed to read planned input file `{}`", input.path))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .with_context(|| format!("failed to hash planned input file `{}`", input.path))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        let digest = hasher.finalize();
        let actual = format!("sha256:{}", output::lower_hex(&digest));
        if input.sha256 != actual {
            return usage(format!(
                "input file hash mismatch for `{}`; create a new plan",
                input.path
            ));
        }
    }
    Ok(())
}

fn validate_request(
    operation: &str,
    request: &PlannedRequest,
    profile: &Profile,
) -> anyhow::Result<AtlassianProduct> {
    let (product, method, route) = match operation {
        "jira.issue.create" => (
            AtlassianProduct::Jira,
            Method::POST,
            Route::Exact("/rest/api/3/issue"),
        ),
        "confluence.page.create" => (
            AtlassianProduct::Confluence,
            Method::POST,
            Route::Exact("/wiki/api/v2/pages"),
        ),
        "confluence.page.update" => (
            AtlassianProduct::Confluence,
            Method::PUT,
            Route::Resource("/wiki/api/v2/pages/"),
        ),
        "confluence.blog.create" => (
            AtlassianProduct::Confluence,
            Method::POST,
            Route::Exact("/wiki/api/v2/blogposts"),
        ),
        "confluence.blog.update" => (
            AtlassianProduct::Confluence,
            Method::PUT,
            Route::Resource("/wiki/api/v2/blogposts/"),
        ),
        _ => return usage(format!("operation `{operation}` is not applicable")),
    };
    if request.method != method.as_str() {
        return usage(format!(
            "operation `{operation}` requires {}, not {}",
            method, request.method
        ));
    }
    let base = match product {
        AtlassianProduct::Jira => profile.jira_api_base_url(),
        AtlassianProduct::Confluence => profile.confluence_api_base_url(),
    };
    let base = Url::parse(&base).context("profile API base URL is invalid")?;
    let url = Url::parse(&request.url).context("plan request URL is invalid")?;
    if !same_origin(&base, &url)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return usage("plan request URL does not match the profile API origin");
    }
    let base_path = base.path().trim_end_matches('/');
    let relative_path = url.path().strip_prefix(base_path).ok_or_else(|| {
        anyhow::Error::new(UsageError(
            "plan request URL escaped the API base path".to_owned(),
        ))
    })?;
    match route {
        Route::Exact(expected) if relative_path != expected => {
            return usage(format!(
                "plan request path `{relative_path}` is not allowed"
            ));
        }
        Route::Resource(prefix) => {
            let resource = relative_path.strip_prefix(prefix).ok_or_else(|| {
                anyhow::Error::new(UsageError(format!(
                    "plan request path `{relative_path}` is not allowed"
                )))
            })?;
            let resource_id = if operation == "confluence.page.update" {
                resource.strip_suffix("/title").unwrap_or(resource)
            } else {
                resource
            };
            let valid = !resource_id.is_empty()
                && resource_id.bytes().all(|byte| byte.is_ascii_digit())
                && (resource == resource_id || resource == format!("{resource_id}/title"));
            if !valid {
                return usage(format!(
                    "plan request resource path `{relative_path}` is not allowed"
                ));
            }
        }
        _ => {}
    }
    validate_query(operation, &url)?;
    Ok(product)
}

fn validate_query(operation: &str, url: &Url) -> anyhow::Result<()> {
    let allowed: &[&str] = match operation {
        "confluence.page.create" => &["private", "root-level"],
        "confluence.blog.create" => &["private"],
        _ => &[],
    };
    for (key, value) in url.query_pairs() {
        if !allowed.contains(&key.as_ref()) || value != "true" {
            return usage(format!("plan request query `{key}={value}` is not allowed"));
        }
    }
    Ok(())
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

enum Route {
    Exact(&'static str),
    Resource(&'static str),
}

fn usage<T>(message: impl Into<String>) -> anyhow::Result<T> {
    Err(anyhow::Error::new(UsageError(message.into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use atla_core::{CredentialStorage, ProfilePolicy};

    fn profile() -> Profile {
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "you@example.com".to_owned(),
            credential_store: CredentialStorage::File,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        }
    }

    fn plan(created_at: DateTime<Utc>, expires_at: DateTime<Utc>) -> OperationPlan {
        let mut plan = OperationPlan {
            schema_version: output::schema::SCHEMA_VERSION,
            plan_version: output::schema::PLAN_VERSION,
            operation: "jira.issue.create".to_owned(),
            profile: "work".to_owned(),
            site: "https://example.atlassian.net".to_owned(),
            requests: vec![PlannedRequest {
                method: "POST".to_owned(),
                url: "https://example.atlassian.net/rest/api/3/issue".to_owned(),
                body: Some(serde_json::json!({"fields": {"summary": "Test"}})),
            }],
            preconditions: Vec::new(),
            unresolved: Vec::new(),
            input_files: Vec::new(),
            mutating: true,
            created_at: created_at.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
            plan_hash: String::new(),
        };
        plan.plan_hash = output::operation_plan_hash(&plan).expect("plan hash");
        plan
    }

    #[test]
    fn rejects_expired_plan_even_with_valid_hash() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("expired.json");
        let plan = plan(
            Utc::now() - Duration::hours(2),
            Utc::now() - Duration::hours(1),
        );
        std::fs::write(&path, serde_json::to_vec(&plan).expect("serialize plan"))
            .expect("write plan");

        let error = load_and_verify(&path).expect_err("expired plan must fail");
        assert!(error.to_string().contains("expired"));
    }

    #[test]
    fn rejects_cross_origin_and_non_allowlisted_paths() {
        let cross_origin = PlannedRequest {
            method: "POST".to_owned(),
            url: "https://evil.example/rest/api/3/issue".to_owned(),
            body: Some(serde_json::json!({})),
        };
        assert!(validate_request("jira.issue.create", &cross_origin, &profile()).is_err());

        let wrong_path = PlannedRequest {
            method: "POST".to_owned(),
            url: "https://example.atlassian.net/rest/api/3/issue/PROJ-1".to_owned(),
            body: Some(serde_json::json!({})),
        };
        assert!(validate_request("jira.issue.create", &wrong_path, &profile()).is_err());
    }

    #[test]
    fn plan_hash_changes_with_request_body() {
        let now = Utc::now();
        let mut plan = plan(now, now + Duration::hours(1));
        let original = plan.plan_hash.clone();
        plan.requests[0].body = Some(serde_json::json!({"fields": {"summary": "Changed"}}));
        assert_ne!(
            output::operation_plan_hash(&plan).expect("new hash"),
            original
        );
    }
}
