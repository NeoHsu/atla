use anyhow::Context;
use atla_core::{ConfigStore, PolicyDecisionSource};
use serde::Serialize;

use crate::cli::{ExplainPolicyArgs, GlobalArgs, OutputFormat};
use crate::error::UsageError;
use crate::operation;
use crate::output;
use crate::output::schema::SCHEMA_VERSION;
use crate::policy;

use super::{OperationView, operation_view};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PolicyExplanation {
    schema_version: u32,
    operation: OperationView,
    profile: Option<String>,
    profile_found: bool,
    policy_mode: Option<String>,
    profile_decision: &'static str,
    matched_pattern: Option<String>,
    global_read_only: bool,
    global_read_only_blocked: bool,
    allowed: bool,
    reason: String,
}

pub fn explain_policy(args: ExplainPolicyArgs, global: &GlobalArgs) -> anyhow::Result<()> {
    let metadata = operation::by_id(&args.operation_id).ok_or_else(|| {
        anyhow::Error::new(UsageError(format!(
            "unknown operation `{}`; run `atla operation list`",
            args.operation_id
        )))
    })?;
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let config = store
        .load_read_only()
        .context("failed to load config for policy explanation")?;

    let mut profile = None;
    let mut profile_found = false;
    let mut policy_mode = None;
    let policy_applies = policy::profile_policy_applies(metadata.id);
    let mut profile_decision = if policy_applies {
        "no-profile"
    } else {
        "not-applicable"
    };
    let mut matched_pattern = None;
    let mut profile_allowed = !policy_applies;
    if let Some((name, selected)) = config.active_profile(global.profile.as_deref()) {
        profile = Some(name.to_owned());
        profile_found = true;
        policy_mode = Some(selected.policy.mode.to_string());
        if policy_applies {
            let decision = selected
                .policy
                .decision(metadata.id, metadata.risk.mutates());
            profile_allowed = decision.allowed;
            profile_decision = match decision.source {
                PolicyDecisionSource::Deny => "deny-rule",
                PolicyDecisionSource::Allow => "allow-rule",
                PolicyDecisionSource::Mode => "mode",
            };
            matched_pattern = decision.matched_pattern;
        }
    }

    let global_read_only_blocked = global.read_only && metadata.risk.mutates();
    let allowed = profile_allowed && !global_read_only_blocked;
    let reason = if global_read_only_blocked {
        "blocked because --read-only rejects mutating operations".to_owned()
    } else if !policy_applies {
        "profile policy does not apply to this local recovery/discovery operation".to_owned()
    } else if !profile_found {
        match global.profile.as_deref() {
            Some(name) => format!("blocked because requested profile `{name}` does not exist"),
            None => "blocked because no active profile is configured".to_owned(),
        }
    } else {
        match profile_decision {
            "deny-rule" => format!(
                "blocked by deny rule `{}`",
                matched_pattern.as_deref().unwrap_or_default()
            ),
            "allow-rule" => format!(
                "allowed by allow rule `{}`",
                matched_pattern.as_deref().unwrap_or_default()
            ),
            "mode" if allowed => "allowed by profile policy mode".to_owned(),
            "mode" => "blocked by read-only profile policy mode".to_owned(),
            _ => "allowed".to_owned(),
        }
    };
    let report = PolicyExplanation {
        schema_version: SCHEMA_VERSION,
        operation: operation_view(metadata),
        profile,
        profile_found,
        policy_mode,
        profile_decision,
        matched_pattern,
        global_read_only: global.read_only,
        global_read_only_blocked,
        allowed,
        reason,
    };

    match global.output {
        Some(OutputFormat::Json) => output::print_json(&report),
        Some(format @ (OutputFormat::Table | OutputFormat::Csv | OutputFormat::Keys)) => {
            output::print_records(
                format,
                &report,
                vec![report.operation.id.to_owned()],
                &[
                    "operation",
                    "risk",
                    "profile",
                    "profile_decision",
                    "read_only_blocked",
                    "allowed",
                    "reason",
                ],
                vec![vec![
                    report.operation.id.to_owned(),
                    report.operation.risk.to_owned(),
                    report.profile.clone().unwrap_or_default(),
                    report.profile_decision.to_owned(),
                    report.global_read_only_blocked.to_string(),
                    report.allowed.to_string(),
                    report.reason.clone(),
                ]],
                None,
            )
        }
        None => {
            println!("Operation: {}", report.operation.id);
            println!("Risk: {}", report.operation.risk);
            println!("Profile: {}", report.profile.as_deref().unwrap_or("none"));
            println!("Profile decision: {}", report.profile_decision);
            if let Some(pattern) = report.matched_pattern.as_deref() {
                println!("Matched pattern: {pattern}");
            }
            println!(
                "Global read-only blocked: {}",
                report.global_read_only_blocked
            );
            println!("Allowed: {}", report.allowed);
            println!("Reason: {}", report.reason);
            Ok(())
        }
    }
}
