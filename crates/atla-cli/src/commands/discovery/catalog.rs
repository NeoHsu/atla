use anyhow::Context;
use serde::Serialize;

use crate::cli::{
    GlobalArgs, OperationAction, OperationCommand, OutputFormat, SchemaAction, SchemaCommand,
};
use crate::error::UsageError;
use crate::operation;
use crate::output;
use crate::output::schema::{Pagination, SCHEMA_VERSION};

use super::{OperationView, operation_view};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationListReport {
    schema_version: u32,
    operations: Vec<OperationView>,
    pagination: Pagination,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaView {
    name: &'static str,
    file: &'static str,
    description: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaListReport {
    schema_version: u32,
    schemas: Vec<SchemaView>,
    pagination: Pagination,
}

struct BundledSchema {
    view: SchemaView,
    content: &'static str,
}

const BUNDLED_SCHEMAS: &[BundledSchema] = &[
    BundledSchema {
        view: SchemaView {
            name: "confluence-page-list-v1",
            file: "confluence-page-list-v1.schema.json",
            description: "Confluence page list output",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/confluence-page-list-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "doctor-v1",
            file: "doctor-v1.schema.json",
            description: "Local and optional network diagnostic report",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/doctor-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "error-v1",
            file: "error-v1.schema.json",
            description: "Structured stderr error envelope",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/error-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "jira-issue-list-v1",
            file: "jira-issue-list-v1.schema.json",
            description: "Jira issue list output",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/jira-issue-list-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "list-v1",
            file: "list-v1.schema.json",
            description: "Shared list pagination contract",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/list-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "mutation-receipt-v1",
            file: "mutation-receipt-v1.schema.json",
            description: "Successful mutation receipt metadata",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/mutation-receipt-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "operation-list-v1",
            file: "operation-list-v1.schema.json",
            description: "Stable operation and safety metadata list",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/operation-list-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "operation-plan-v1",
            file: "operation-plan-v1.schema.json",
            description: "Validated mutation plan",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/operation-plan-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "policy-explanation-v1",
            file: "policy-explanation-v1.schema.json",
            description: "Profile and global read-only policy decision",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/policy-explanation-v1.schema.json"
        )),
    },
    BundledSchema {
        view: SchemaView {
            name: "schema-list-v1",
            file: "schema-list-v1.schema.json",
            description: "Bundled public schema discovery list",
        },
        content: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/schemas/schema-list-v1.schema.json"
        )),
    },
];

pub fn operation(command: OperationCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        OperationAction::List => list_operations(global),
    }
}

fn list_operations(global: &GlobalArgs) -> anyhow::Result<()> {
    let operations = operation::catalog()
        .iter()
        .copied()
        .map(operation_view)
        .collect::<Vec<_>>();
    let report = OperationListReport {
        schema_version: SCHEMA_VERSION,
        operations,
        pagination: complete_pagination(),
    };
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        &report,
        report
            .operations
            .iter()
            .map(|operation| operation.id.to_owned())
            .collect(),
        &[
            "id",
            "method",
            "risk",
            "mutating",
            "paginated",
            "dry_run",
            "retry_safe",
        ],
        report
            .operations
            .iter()
            .map(|operation| {
                vec![
                    operation.id.to_owned(),
                    operation.method.unwrap_or("LOCAL").to_owned(),
                    operation.risk.to_owned(),
                    operation.mutating.to_string(),
                    operation.paginated.to_string(),
                    operation.dry_run.to_string(),
                    operation.retry_safe.to_string(),
                ]
            })
            .collect(),
        None,
    )
}

pub fn schema(command: SchemaCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        SchemaAction::List => list_schemas(global),
        SchemaAction::Print { name } => print_schema(&name, global),
    }
}

fn list_schemas(global: &GlobalArgs) -> anyhow::Result<()> {
    let schemas = BUNDLED_SCHEMAS
        .iter()
        .map(|schema| schema.view.clone())
        .collect::<Vec<_>>();
    let report = SchemaListReport {
        schema_version: SCHEMA_VERSION,
        schemas,
        pagination: complete_pagination(),
    };
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        &report,
        report
            .schemas
            .iter()
            .map(|schema| schema.name.to_owned())
            .collect(),
        &["name", "file", "description"],
        report
            .schemas
            .iter()
            .map(|schema| {
                vec![
                    schema.name.to_owned(),
                    schema.file.to_owned(),
                    schema.description.to_owned(),
                ]
            })
            .collect(),
        None,
    )
}

fn print_schema(name: &str, global: &GlobalArgs) -> anyhow::Result<()> {
    if global
        .output
        .is_some_and(|format| format != OutputFormat::Json)
    {
        return Err(anyhow::Error::new(UsageError(
            "schema print supports only --output json".to_owned(),
        )));
    }
    let normalized = name.strip_suffix(".schema.json").unwrap_or(name);
    let schema = BUNDLED_SCHEMAS
        .iter()
        .find(|schema| schema.view.name == normalized)
        .ok_or_else(|| {
            anyhow::Error::new(UsageError(format!(
                "unknown schema `{name}`; run `atla schema list`"
            )))
        })?;
    serde_json::from_str::<serde_json::Value>(schema.content)
        .with_context(|| format!("bundled schema `{normalized}` is invalid"))?;
    output::print_raw(schema.content)
}

fn complete_pagination() -> Pagination {
    Pagination {
        is_last: true,
        next_page_token: None,
        next_command: None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use super::*;

    #[test]
    fn bundled_schema_registry_matches_docs_directory() {
        let schema_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/schemas");
        let mut files = BTreeSet::new();
        for entry in std::fs::read_dir(schema_dir).expect("read schema directory") {
            let path = entry.expect("schema directory entry").path();
            if let Some(name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".schema.json"))
            {
                files.insert(name.to_owned());
            }
        }
        let registered = BUNDLED_SCHEMAS
            .iter()
            .map(|schema| schema.view.name.to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(registered, files);
    }

    #[test]
    fn bundled_schemas_are_valid_json() {
        for schema in BUNDLED_SCHEMAS {
            serde_json::from_str::<serde_json::Value>(schema.content)
                .unwrap_or_else(|error| panic!("{}: {error}", schema.view.name));
        }
    }

    #[test]
    fn operation_view_exposes_retry_and_mutation_semantics() {
        let operation =
            operation_view(operation::by_id("jira.issue.create").expect("registered operation"));
        assert!(operation.mutating);
        assert!(!operation.retry_safe);
        assert_eq!(operation.risk, "write");
    }

    #[test]
    fn policy_mode_display_is_stable() {
        assert_eq!(atla_core::PolicyMode::ReadOnly.to_string(), "read-only");
    }
}
