use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use comfy_table::{Table, presets::NOTHING};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::cli::OutputFormat;
use crate::error::UsageError;

pub mod schema;

/// Zero means unlimited. The CLI configures this once before dispatch.
static MAX_OUTPUT_BYTES: AtomicU64 = AtomicU64::new(0);
static EXECUTION_CONTEXT: OnceLock<Mutex<ExecutionContext>> = OnceLock::new();
static PLAN_OUTPUT: OnceLock<Mutex<Option<PlanOutput>>> = OnceLock::new();
static PLAN_INPUTS: OnceLock<Mutex<Vec<schema::InputFileDigest>>> = OnceLock::new();

#[derive(Default)]
struct ExecutionContext {
    operation: Option<String>,
    profile: Option<String>,
    mutating: bool,
    dry_run: bool,
}

struct PlanOutput {
    path: PathBuf,
    expires_in_seconds: u64,
}

pub fn configure_operation(operation: impl Into<String>, mutating: bool, dry_run: bool) {
    let mut context = EXECUTION_CONTEXT
        .get_or_init(|| Mutex::new(ExecutionContext::default()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *context = ExecutionContext {
        operation: Some(operation.into()),
        profile: None,
        mutating,
        dry_run,
    };
}

pub fn configure_plan_output(path: PathBuf, expires_in_seconds: u64) {
    let mut output = PLAN_OUTPUT
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *output = Some(PlanOutput {
        path,
        expires_in_seconds,
    });
    PLAN_INPUTS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}

pub fn register_plan_input(path: &std::path::Path) -> anyhow::Result<()> {
    let canonical = path.canonicalize().map_err(|error| {
        anyhow::anyhow!("failed to resolve input file `{}`: {error}", path.display())
    })?;
    let bytes = std::fs::read(&canonical).map_err(|error| {
        anyhow::anyhow!(
            "failed to hash input file `{}`: {error}",
            canonical.display()
        )
    })?;
    let input = schema::InputFileDigest {
        path: canonical.to_string_lossy().into_owned(),
        sha256: format!("sha256:{:x}", Sha256::digest(bytes)),
    };
    let mut inputs = PLAN_INPUTS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !inputs.iter().any(|existing| existing.path == input.path) {
        inputs.push(input);
    }
    Ok(())
}

pub fn configure_profile(profile: &str) {
    let mut context = EXECUTION_CONTEXT
        .get_or_init(|| Mutex::new(ExecutionContext::default()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    context.profile = Some(profile.to_owned());
}

pub fn configure_max_bytes(max_bytes: Option<u64>) {
    MAX_OUTPUT_BYTES.store(max_bytes.unwrap_or(0), Ordering::Relaxed);
}

fn print_bounded(rendered: String) -> anyhow::Result<()> {
    let maximum = MAX_OUTPUT_BYTES.load(Ordering::Relaxed);
    let actual = rendered.len() as u64;
    if maximum != 0 && actual > maximum {
        return Err(anyhow::Error::new(UsageError(format!(
            "output requires {actual} bytes, exceeding --max-bytes {maximum}"
        ))));
    }
    if !rendered.is_empty() {
        println!("{rendered}");
    }
    Ok(())
}

pub fn print_json<T: Serialize + ?Sized>(value: &T) -> anyhow::Result<()> {
    let mut value = serde_json::to_value(value)?;
    if let serde_json::Value::Object(object) = &mut value {
        object
            .entry("schemaVersion")
            .or_insert_with(|| schema::SCHEMA_VERSION.into());
        let context = EXECUTION_CONTEXT
            .get_or_init(|| Mutex::new(ExecutionContext::default()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if context.mutating
            && !context.dry_run
            && let (Some(operation), Some(profile)) =
                (context.operation.as_deref(), context.profile.as_ref())
        {
            let target = infer_target(object);
            object
                .entry("operation")
                .or_insert_with(|| operation.into());
            object
                .entry("profile")
                .or_insert_with(|| profile.clone().into());
            object.entry("target").or_insert(target);
            object.entry("requestId").or_insert(serde_json::Value::Null);
            object.entry("completedAt").or_insert_with(|| {
                chrono::Utc::now()
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                    .into()
            });
        }
    }
    print_bounded(serde_json::to_string_pretty(&value)?)
}

fn infer_target(object: &serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
    ["key", "id", "deleted", "updated", "created"]
        .into_iter()
        .find_map(|key| object.get(key))
        .and_then(|value| match value {
            serde_json::Value::String(_) | serde_json::Value::Number(_) => Some(value.clone()),
            _ => None,
        })
        .unwrap_or(serde_json::Value::Null)
}

/// Print the JSON body a --dry-run mutation would send, so callers can verify
/// field assembly and Markdown conversion before executing.
pub fn print_dry_run_body<T: Serialize + ?Sized>(body: &T) -> anyhow::Result<()> {
    print_bounded(format!(
        "Request body:\n{}",
        serde_json::to_string_pretty(body)?
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn print_operation_plan(
    operation: &'static str,
    profile: &str,
    site: &str,
    method: &str,
    url: String,
    body: Option<serde_json::Value>,
    preconditions: Vec<String>,
    unresolved: Vec<String>,
) -> anyhow::Result<()> {
    let plan_output = PLAN_OUTPUT
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let expires_in_seconds = plan_output
        .as_ref()
        .map_or(3600, |output| output.expires_in_seconds);
    let created_at = chrono::Utc::now();
    let expires_at = created_at + chrono::Duration::seconds(expires_in_seconds as i64);
    let mut plan = schema::OperationPlan {
        schema_version: schema::SCHEMA_VERSION,
        plan_version: schema::PLAN_VERSION,
        operation: operation.to_owned(),
        profile: profile.to_owned(),
        site: site.to_owned(),
        requests: vec![schema::PlannedRequest {
            method: method.to_owned(),
            url,
            body,
        }],
        preconditions,
        unresolved,
        input_files: PLAN_INPUTS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone(),
        mutating: true,
        created_at: created_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        expires_at: expires_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        plan_hash: String::new(),
    };
    plan.plan_hash = operation_plan_hash(&plan)?;

    if let Some(output) = plan_output.as_ref() {
        let output_path = normalized_write_path(&output.path)?;
        if plan
            .input_files
            .iter()
            .any(|input| std::path::Path::new(&input.path) == output_path)
        {
            return Err(anyhow::Error::new(UsageError(format!(
                "plan output `{}` cannot overwrite an input file",
                output.path.display()
            ))));
        }
        let rendered = serde_json::to_string_pretty(&plan)?;
        ensure_within_byte_budget(&rendered)?;
        atla_core::secure_file::atomic_write(&output.path, rendered.as_bytes())?;
        print_json(&serde_json::json!({
            "operation": plan.operation,
            "planFile": output.path,
            "planHash": plan.plan_hash,
            "expiresAt": plan.expires_at,
        }))
    } else {
        drop(plan_output);
        print_json(&plan)
    }
}

fn normalized_write_path(path: &std::path::Path) -> anyhow::Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .map_err(|error| anyhow::anyhow!("failed to resolve `{}`: {error}", path.display()));
    }
    if path.file_name().is_none() {
        anyhow::bail!("plan output must name a file");
    }
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

pub fn operation_plan_hash(plan: &schema::OperationPlan) -> anyhow::Result<String> {
    let mut unsigned = plan.clone();
    unsigned.plan_hash.clear();
    let digest = Sha256::digest(serde_json::to_vec(&unsigned)?);
    Ok(format!("sha256:{digest:x}"))
}

fn ensure_within_byte_budget(rendered: &str) -> anyhow::Result<()> {
    let maximum = MAX_OUTPUT_BYTES.load(Ordering::Relaxed);
    let actual = rendered.len() as u64;
    if maximum != 0 && actual > maximum {
        return Err(anyhow::Error::new(UsageError(format!(
            "output requires {actual} bytes, exceeding --max-bytes {maximum}"
        ))));
    }
    Ok(())
}

pub fn print_records<T: Serialize + ?Sized>(
    format: OutputFormat,
    json: &T,
    keys: Vec<String>,
    headers: &[&str],
    rows: Vec<Vec<String>>,
    footer: Option<String>,
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => print_json(json),
        OutputFormat::Keys => print_keys(keys),
        OutputFormat::Csv => print_csv(headers, rows),
        OutputFormat::Table => print_table(headers, rows, footer),
    }
}

pub fn print_keys(keys: Vec<String>) -> anyhow::Result<()> {
    print_bounded(keys.join("\n"))
}

pub fn print_csv(headers: &[&str], rows: Vec<Vec<String>>) -> anyhow::Result<()> {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(headers.join(","));
    lines.extend(rows.into_iter().map(|row| {
        row.iter()
            .map(|value| csv_cell(value))
            .collect::<Vec<_>>()
            .join(",")
    }));
    print_bounded(lines.join("\n"))
}

pub fn print_table(
    headers: &[&str],
    rows: Vec<Vec<String>>,
    footer: Option<String>,
) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.load_preset(NOTHING);
    table.set_header(headers.iter().map(|header| header.to_ascii_uppercase()));
    for row in rows {
        table.add_row(row);
    }
    let mut rendered = table.to_string();
    if let Some(footer) = footer {
        rendered.push_str("\n\n");
        rendered.push_str(&footer);
    }
    print_bounded(rendered)
}

pub fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}
