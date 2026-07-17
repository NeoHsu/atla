use anyhow::Context;
use atla_core::ConfigStore;

use crate::cli::GlobalArgs;
use crate::error::UsageError;
use crate::operation::OperationMetadata;

/// Enforces profile allow/deny rules without loading credentials or making a
/// network request. Local auth/config management remains available so a user
/// can recover from an overly restrictive profile policy.
pub fn enforce_profile_policy(
    global: &GlobalArgs,
    operation: OperationMetadata,
) -> anyhow::Result<()> {
    if global.dry_run
        || operation.id.starts_with("auth.")
        || operation.id.starts_with("config.")
        || operation.id == "plan.apply"
        || operation.id == "completion"
    {
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
