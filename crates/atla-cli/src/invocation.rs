use std::ops::Deref;

use crate::cli::GlobalArgs;
use crate::operation::OperationMetadata;
use crate::output::OutputSession;

/// State owned by one parsed CLI invocation.
///
/// `GlobalArgs` remains the immutable command-line configuration while mutable
/// rendering and plan-building state lives in an invocation-local session.
#[derive(Debug)]
pub struct Invocation {
    global: GlobalArgs,
    output: OutputSession,
}

impl Invocation {
    pub fn new(global: GlobalArgs, operation: OperationMetadata) -> Self {
        let output = OutputSession::new(global.max_bytes);
        output.configure_operation(operation.id, operation.risk.mutates(), global.dry_run);
        Self { global, output }
    }

    pub fn args(&self) -> &GlobalArgs {
        &self.global
    }

    pub fn output(&self) -> &OutputSession {
        &self.output
    }
}

impl Deref for Invocation {
    type Target = GlobalArgs;

    fn deref(&self) -> &Self::Target {
        self.args()
    }
}
