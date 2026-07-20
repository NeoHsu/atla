use anyhow::Context;
use atla_core::ConfigStore;

use crate::cli::GlobalArgs;
use crate::error::UsageError;
use crate::operation::OperationMetadata;

/// Whether an operation is governed by profile allow/deny/mode rules.
/// Local recovery and discovery commands deliberately bypass profile policy.
pub fn profile_policy_applies(operation_id: &str) -> bool {
    !operation_id.starts_with("auth.")
        && !operation_id.starts_with("config.")
        && !operation_id.starts_with("operation.")
        && !operation_id.starts_with("schema.")
        && !matches!(
            operation_id,
            "doctor" | "explain-policy" | "plan.apply" | "completion"
        )
}

/// Enforces profile allow/deny rules without loading credentials or making a
/// network request. Local auth/config management remains available so a user
/// can recover from an overly restrictive profile policy.
pub fn enforce_profile_policy(
    global: &GlobalArgs,
    operation: OperationMetadata,
) -> anyhow::Result<()> {
    if global.dry_run || !profile_policy_applies(operation.id) {
        return Ok(());
    }

    let store = ConfigStore::default_store().context("failed to find config location")?;
    let config = store
        .load_read_only()
        .context("failed to load config for operation policy")?;
    let Some((profile_name, profile)) = config.active_profile(global.profile.as_deref()) else {
        return Ok(());
    };
    if profile
        .policy
        .allows(operation.id, operation.risk.mutates())
    {
        return Ok(());
    }

    Err(anyhow::Error::new(UsageError(format!(
        "operation `{}` is blocked by policy for profile `{profile_name}`",
        operation.id
    ))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_policy_applies_only_to_product_operations() {
        assert!(profile_policy_applies("jira.issue.view"));
        assert!(profile_policy_applies("confluence.page.update"));
        for operation in [
            "auth.login",
            "config.set",
            "doctor",
            "explain-policy",
            "operation.list",
            "schema.print",
            "plan.apply",
            "completion",
        ] {
            assert!(!profile_policy_applies(operation), "{operation}");
        }
    }
}
