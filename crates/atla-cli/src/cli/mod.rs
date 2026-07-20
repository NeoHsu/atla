use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(name = "atla", version, about = "Unified Atlassian CLI")]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// Output format
    #[arg(short, long, value_enum, global = true)]
    pub output: Option<OutputFormat>,

    /// Auth/config profile to use
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Log HTTP requests to stderr
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Print the request instead of executing the mutation
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Never prompt; fail instead (for scripts and CI)
    #[arg(long, global = true)]
    pub no_input: bool,

    /// Reject every command that can modify local or remote state
    #[arg(long, global = true, env = "ATLA_READ_ONLY")]
    pub read_only: bool,

    /// Stop automatic pagination after this many API pages
    #[arg(long, global = true, value_parser = clap::value_parser!(u32).range(1..))]
    pub max_pages: Option<u32>,

    /// Return at most this many records from a list operation
    #[arg(long, global = true, value_parser = clap::value_parser!(u32).range(1..))]
    pub max_items: Option<u32>,

    /// Refuse to print JSON output larger than this many bytes (requires --output json)
    #[arg(long, global = true, value_parser = clap::value_parser!(u64).range(1..))]
    pub max_bytes: Option<u64>,

    /// Per-request timeout in seconds (including uploads and downloads)
    #[arg(long, global = true, value_parser = clap::value_parser!(u64).range(1..))]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
    Keys,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AuthStorage {
    Keyring,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BodyRepresentation {
    Storage,
    Wiki,
    AtlasDocFormat,
    Markdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AttachmentMode {
    Auto,
    Link,
    Embed,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ContentViewFormat {
    Markdown,
    Storage,
    AtlasDocFormat,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Manage authentication profiles and API tokens
    Auth(AuthCommand),
    /// Read and write config values and command aliases
    Config(ConfigCommand),
    /// Jira projects, issues, boards, and sprints
    Jira(JiraCommand),
    /// Confluence spaces, pages, blogs, search, and attachments
    Confluence(ConfluenceCommand),
    /// Diagnose local configuration, credentials, policy, and optional site reachability
    Doctor(DoctorArgs),
    /// Explain why an operation is allowed or blocked by current policy
    ExplainPolicy(ExplainPolicyArgs),
    /// Discover stable operation IDs and safety metadata
    Operation(OperationCommand),
    /// Discover and print bundled public JSON schemas
    Schema(SchemaCommand),
    /// Build a validated, expiring mutation plan without network access
    Plan {
        /// Write the plan to this file
        #[arg(long, global = true)]
        out: Option<PathBuf>,
        /// Plan lifetime in seconds
        #[arg(long, global = true, default_value_t = 3600, value_parser = clap::value_parser!(u64).range(1..=86400))]
        expires_in: u64,
        #[command(subcommand)]
        command: PlannableCommand,
    },
    /// Validate and execute a saved mutation plan
    Apply {
        /// Plan JSON file
        plan: PathBuf,
        /// Confirm execution of the saved mutation
        #[arg(long)]
        yes: bool,
    },
    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

mod auth;
mod config;
mod confluence;
mod discovery;
mod jira;
mod plan;

pub use auth::*;
pub use config::*;
pub use confluence::*;
pub use discovery::*;
pub use jira::*;
pub use plan::*;

#[cfg(test)]
mod tests;
