use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use comfy_table::{Table, presets::NOTHING};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::cli::OutputFormat;
use crate::error::UsageError;
use crate::operation::OperationId;

pub mod schema;

pub(crate) const MAX_PLAN_BYTES: u64 = 1024 * 1024;

/// Per-invocation output state.
///
/// Keeping receipts, byte budgets, and plan inputs on the invocation prevents
/// parallel commands and tests in the same process from leaking state into one
/// another.
#[derive(Debug)]
pub struct OutputSession {
    max_output_bytes: u64,
    execution: Mutex<ExecutionContext>,
    plan_output: Mutex<Option<PlanOutput>>,
    plan_inputs: Mutex<Vec<schema::InputFileDigest>>,
}

#[derive(Debug, Default)]
struct ExecutionContext {
    operation: Option<OperationId>,
    profile: Option<String>,
    mutating: bool,
    dry_run: bool,
}

#[derive(Debug)]
struct PlanOutput {
    path: PathBuf,
    expires_in_seconds: u64,
}

impl Default for OutputSession {
    fn default() -> Self {
        Self::new(None)
    }
}

impl OutputSession {
    pub fn new(max_output_bytes: Option<u64>) -> Self {
        Self {
            max_output_bytes: max_output_bytes.unwrap_or(0),
            execution: Mutex::new(ExecutionContext::default()),
            plan_output: Mutex::new(None),
            plan_inputs: Mutex::new(Vec::new()),
        }
    }

    pub fn configure_operation(&self, operation: OperationId, mutating: bool, dry_run: bool) {
        let mut context = self
            .execution
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *context = ExecutionContext {
            operation: Some(operation),
            profile: None,
            mutating,
            dry_run,
        };
    }

    pub fn configure_plan_output(&self, path: PathBuf, expires_in_seconds: u64) {
        let mut output = self
            .plan_output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *output = Some(PlanOutput {
            path,
            expires_in_seconds,
        });
        self.plan_inputs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    pub fn register_plan_input(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let canonical = path.canonicalize().map_err(|error| {
            anyhow::anyhow!("failed to resolve input file `{}`: {error}", path.display())
        })?;
        let bytes = std::fs::read(&canonical).map_err(|error| {
            anyhow::anyhow!(
                "failed to hash input file `{}`: {error}",
                canonical.display()
            )
        })?;
        let digest = Sha256::digest(bytes);
        let input = schema::InputFileDigest {
            path: canonical.to_string_lossy().into_owned(),
            sha256: format!("sha256:{}", lower_hex(&digest)),
        };
        let mut inputs = self
            .plan_inputs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !inputs.iter().any(|existing| existing.path == input.path) {
            inputs.push(input);
        }
        Ok(())
    }

    pub fn configure_profile(&self, profile: &str) {
        let mut context = self
            .execution
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        context.profile = Some(profile.to_owned());
    }

    fn ensure_within_byte_budget(&self, rendered: &str) -> anyhow::Result<()> {
        let actual = rendered.len() as u64;
        if self.max_output_bytes != 0 && actual > self.max_output_bytes {
            return Err(anyhow::Error::new(UsageError(format!(
                "output requires {actual} bytes, exceeding --max-bytes {}",
                self.max_output_bytes
            ))));
        }
        Ok(())
    }

    fn write_bounded(&self, rendered: &str, writer: &mut dyn Write) -> anyhow::Result<()> {
        self.ensure_within_byte_budget(rendered)?;
        if !rendered.is_empty() {
            writeln!(writer, "{rendered}")?;
        }
        Ok(())
    }

    fn print_bounded(&self, rendered: String) -> anyhow::Result<()> {
        let stdout = std::io::stdout();
        self.write_bounded(&rendered, &mut stdout.lock())
    }

    pub fn print_raw(&self, value: &str) -> anyhow::Result<()> {
        self.print_bounded(value.trim_end().to_owned())
    }

    fn render_json<T: Serialize + ?Sized>(&self, value: &T) -> anyhow::Result<String> {
        let mut value = serde_json::to_value(value)?;
        if let serde_json::Value::Object(object) = &mut value {
            object
                .entry("schemaVersion")
                .or_insert_with(|| schema::SCHEMA_VERSION.into());
            let context = self
                .execution
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if context.mutating
                && !context.dry_run
                && let (Some(operation), Some(profile)) =
                    (context.operation, context.profile.as_ref())
            {
                let target = infer_target(object);
                object
                    .entry("operation")
                    .or_insert_with(|| operation.as_str().into());
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
        Ok(serde_json::to_string_pretty(&value)?)
    }

    pub fn print_json<T: Serialize + ?Sized>(&self, value: &T) -> anyhow::Result<()> {
        self.print_bounded(self.render_json(value)?)
    }

    /// Print the JSON body a --dry-run mutation would send, so callers can
    /// verify field assembly and Markdown conversion before executing.
    pub fn print_dry_run_body<T: Serialize + ?Sized>(&self, body: &T) -> anyhow::Result<()> {
        self.print_bounded(format!(
            "Request body:\n{}",
            serde_json::to_string_pretty(body)?
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn print_operation_plan(
        &self,
        operation: OperationId,
        profile: &str,
        site: &str,
        method: &str,
        url: String,
        body: Option<serde_json::Value>,
        preconditions: Vec<String>,
        unresolved: Vec<String>,
    ) -> anyhow::Result<()> {
        let plan_output = self
            .plan_output
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
            operation: operation.as_str().to_owned(),
            profile: profile.to_owned(),
            site: site.to_owned(),
            requests: vec![schema::PlannedRequest {
                method: method.to_owned(),
                url,
                body,
            }],
            preconditions,
            unresolved,
            input_files: self
                .plan_inputs
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
            let plan_bytes = rendered.len() as u64;
            if plan_bytes > MAX_PLAN_BYTES {
                return Err(anyhow::Error::new(UsageError(format!(
                    "plan is {plan_bytes} bytes; maximum is {MAX_PLAN_BYTES}"
                ))));
            }
            self.ensure_within_byte_budget(&rendered)?;
            atla_core::secure_file::atomic_write(&output.path, rendered.as_bytes())?;
            self.print_json(&serde_json::json!({
                "operation": plan.operation,
                "planFile": output.path,
                "planHash": plan.plan_hash,
                "expiresAt": plan.expires_at,
            }))
        } else {
            drop(plan_output);
            self.print_json(&plan)
        }
    }

    pub fn print_records<T: Serialize + ?Sized>(
        &self,
        format: OutputFormat,
        json: &T,
        keys: Vec<String>,
        headers: &[&str],
        rows: Vec<Vec<String>>,
        footer: Option<String>,
    ) -> anyhow::Result<()> {
        match format {
            OutputFormat::Json => self.print_json(json),
            OutputFormat::Keys => self.print_keys(keys),
            OutputFormat::Csv => self.print_csv(headers, rows),
            OutputFormat::Table => self.print_table(headers, rows, footer),
        }
    }

    pub fn print_keys(&self, keys: Vec<String>) -> anyhow::Result<()> {
        self.print_bounded(keys.join("\n"))
    }

    pub fn print_csv(&self, headers: &[&str], rows: Vec<Vec<String>>) -> anyhow::Result<()> {
        let mut lines = Vec::with_capacity(rows.len() + 1);
        lines.push(headers.join(","));
        lines.extend(rows.into_iter().map(|row| {
            row.iter()
                .map(|value| csv_cell(value))
                .collect::<Vec<_>>()
                .join(",")
        }));
        self.print_bounded(lines.join("\n"))
    }

    pub fn print_table(
        &self,
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
        self.print_bounded(rendered)
    }
}

pub(crate) fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
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
    Ok(format!("sha256:{}", lower_hex(&digest)))
}

pub fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::{OutputSession, lower_hex};

    #[test]
    fn lower_hex_preserves_leading_zeroes() {
        assert_eq!(lower_hex(&[0x00, 0x0f, 0x10, 0xab, 0xff]), "000f10abff");
    }

    #[test]
    fn sha256_digest_format_is_stable() {
        let digest = Sha256::digest(b"abc");
        assert_eq!(
            lower_hex(&digest),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sessions_keep_receipt_context_isolated() {
        let first = OutputSession::new(None);
        first.configure_operation(
            crate::operation::OperationId::JIRA_ISSUE_CREATE,
            true,
            false,
        );
        first.configure_profile("work");
        let second = OutputSession::new(None);

        let first_json = first
            .render_json(&serde_json::json!({"key": "PROJ-1"}))
            .expect("render first session");
        let second_json = second
            .render_json(&serde_json::json!({"key": "PROJ-1"}))
            .expect("render second session");

        assert!(first_json.contains("\"operation\": \"jira.issue.create\""));
        assert!(first_json.contains("\"profile\": \"work\""));
        assert!(!second_json.contains("\"operation\""));
        assert!(!second_json.contains("\"profile\""));
    }

    #[test]
    fn byte_budget_is_per_session() {
        let bounded = OutputSession::new(Some(3));
        let unbounded = OutputSession::new(None);

        assert!(bounded.ensure_within_byte_budget("four").is_err());
        assert!(unbounded.ensure_within_byte_budget("four").is_ok());
    }
}
