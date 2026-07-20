mod catalog;
mod doctor;
mod policy;

use serde::Serialize;

use crate::operation::{OperationMetadata, OperationRisk};

pub use catalog::{operation, schema};
pub use doctor::doctor;
pub use policy::explain_policy;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationView {
    id: &'static str,
    method: Option<&'static str>,
    risk: &'static str,
    mutating: bool,
    paginated: bool,
    dry_run: bool,
    retry_safe: bool,
}

fn operation_view(metadata: OperationMetadata) -> OperationView {
    OperationView {
        id: metadata.id,
        method: metadata.method,
        risk: match metadata.risk {
            OperationRisk::Read => "read",
            OperationRisk::Write => "write",
            OperationRisk::Destructive => "destructive",
        },
        mutating: metadata.risk.mutates(),
        paginated: metadata.paginated,
        dry_run: metadata.dry_run,
        retry_safe: metadata.is_retry_safe(),
    }
}
