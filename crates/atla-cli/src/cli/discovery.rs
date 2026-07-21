use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Also test unauthenticated site reachability and cloud-ID discovery
    #[arg(long)]
    pub network: bool,
    /// Exact atla-cli skill version to compare with this binary
    #[arg(long, value_name = "VERSION")]
    pub skill_version: Option<semver::Version>,
}

#[derive(Debug, Args)]
pub struct ExplainPolicyArgs {
    /// Stable operation ID to evaluate (for example, jira.issue.create)
    pub operation_id: String,
}

#[derive(Debug, Args)]
pub struct OperationCommand {
    #[command(subcommand)]
    pub action: OperationAction,
}

#[derive(Debug, Subcommand)]
pub enum OperationAction {
    /// List stable operation IDs and safety metadata
    List,
}

#[derive(Debug, Args)]
pub struct SchemaCommand {
    #[command(subcommand)]
    pub action: SchemaAction,
}

#[derive(Debug, Subcommand)]
pub enum SchemaAction {
    /// List bundled public JSON schemas
    List,
    /// Print one bundled JSON schema exactly
    Print {
        /// Schema name from `atla schema list` (for example, error-v1)
        name: String,
    },
}
