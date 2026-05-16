use clap::{Args, Parser, Subcommand, ValueEnum};

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
    #[arg(short, long, value_enum)]
    pub output: Option<OutputFormat>,

    #[arg(long)]
    pub profile: Option<String>,

    #[arg(long)]
    pub verbose: bool,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub no_input: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
    Keys,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Auth(AuthCommand),
    Config(ConfigCommand),
    Jira(JiraCommand),
    Confluence(ConfluenceCommand),
}

#[derive(Debug, Args)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub action: AuthAction,
}

#[derive(Debug, Subcommand)]
pub enum AuthAction {
    Login {
        #[arg(long)]
        instance: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        token: Option<String>,
    },
    Logout,
    Status,
    Switch {
        profile: String,
    },
}

#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    Set { key: String, value: String },
    Get { key: String },
    List,
}

#[derive(Debug, Args)]
pub struct JiraCommand {
    #[command(subcommand)]
    pub resource: JiraResource,
}

#[derive(Debug, Subcommand)]
pub enum JiraResource {
    Issue,
    Project,
    Sprint,
    Board,
    Search { jql: String },
}

#[derive(Debug, Args)]
pub struct ConfluenceCommand {
    #[command(subcommand)]
    pub resource: ConfluenceResource,
}

#[derive(Debug, Subcommand)]
pub enum ConfluenceResource {
    Page,
    Space,
    Blog,
    Search { cql: String },
    Attachment,
}
