use std::path::PathBuf;

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
    #[arg(short, long, value_enum, global = true)]
    pub output: Option<OutputFormat>,

    #[arg(long, global = true)]
    pub profile: Option<String>,

    #[arg(long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true)]
    pub dry_run: bool,

    #[arg(long, global = true)]
    pub no_input: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
    Keys,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BodyRepresentation {
    Storage,
    Wiki,
    AtlasDocFormat,
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
    Issue(IssueCommand),
    Project(ProjectCommand),
    Sprint,
    Board,
    Search {
        jql: String,
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
}

#[derive(Debug, Args)]
pub struct IssueCommand {
    #[command(subcommand)]
    pub action: IssueAction,
}

#[derive(Debug, Subcommand)]
pub enum IssueAction {
    List {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        jql: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    Create {
        #[arg(long)]
        project: String,
        #[arg(long = "type")]
        issue_type: String,
        #[arg(long)]
        summary: String,
        #[arg(long, conflicts_with = "description_file")]
        description: Option<String>,
        #[arg(long)]
        description_file: Option<PathBuf>,
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    Update {
        key: String,
        #[arg(long)]
        summary: Option<String>,
        #[arg(long, conflicts_with = "description_file")]
        description: Option<String>,
        #[arg(long)]
        description_file: Option<PathBuf>,
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    View {
        key: String,
    },
    Transition {
        key: String,
        #[arg(long)]
        to: Option<String>,
    },
    Comment {
        #[command(subcommand)]
        action: IssueCommentAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum IssueCommentAction {
    Add {
        key: String,
        #[arg(conflicts_with = "body_file")]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
    List {
        key: String,
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
}

#[derive(Debug, Args)]
pub struct ProjectCommand {
    #[command(subcommand)]
    pub action: ProjectAction,
}

#[derive(Debug, Subcommand)]
pub enum ProjectAction {
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    View {
        key: String,
    },
}

#[derive(Debug, Args)]
pub struct ConfluenceCommand {
    #[command(subcommand)]
    pub resource: ConfluenceResource,
}

#[derive(Debug, Subcommand)]
pub enum ConfluenceResource {
    Page(PageCommand),
    Space(SpaceCommand),
    Blog(BlogCommand),
    Search {
        cql: String,
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
    Attachment(AttachmentCommand),
}

#[derive(Debug, Args)]
pub struct PageCommand {
    #[command(subcommand)]
    pub action: PageAction,
}

#[derive(Debug, Subcommand)]
pub enum PageAction {
    Create {
        #[arg(short = 's', long)]
        space: Option<String>,
        #[arg(long)]
        space_id: Option<String>,
        #[arg(long)]
        title: String,
        #[arg(long, conflicts_with = "root_level")]
        parent: Option<String>,
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
        #[arg(long)]
        draft: bool,
        #[arg(long)]
        private: bool,
        #[arg(long, conflicts_with = "parent")]
        root_level: bool,
    },
    List {
        #[arg(short = 's', long)]
        space: Option<String>,
        #[arg(long)]
        space_id: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
    View {
        id: String,
    },
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
        #[arg(long)]
        version: Option<u64>,
        #[arg(long)]
        message: Option<String>,
        #[arg(long)]
        draft: bool,
    },
    Delete {
        id: String,
        #[arg(long)]
        purge: bool,
        #[arg(long)]
        draft: bool,
        #[arg(long)]
        yes: bool,
    },
    Move {
        id: String,
        #[arg(long)]
        parent: String,
    },
}

#[derive(Debug, Args)]
pub struct BlogCommand {
    #[command(subcommand)]
    pub action: BlogAction,
}

#[derive(Debug, Subcommand)]
pub enum BlogAction {
    Create {
        #[arg(short = 's', long)]
        space: Option<String>,
        #[arg(long)]
        space_id: Option<String>,
        #[arg(long)]
        title: String,
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
        #[arg(long)]
        draft: bool,
        #[arg(long)]
        private: bool,
    },
    List {
        #[arg(short = 's', long)]
        space: Option<String>,
        #[arg(long)]
        space_id: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
    View {
        id: String,
    },
}

#[derive(Debug, Args)]
pub struct SpaceCommand {
    #[command(subcommand)]
    pub action: SpaceAction,
}

#[derive(Debug, Subcommand)]
pub enum SpaceAction {
    List {
        #[arg(long)]
        key: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
    View {
        key: String,
    },
}

#[derive(Debug, Args)]
pub struct AttachmentCommand {
    #[command(subcommand)]
    pub action: AttachmentAction,
}

#[derive(Debug, Subcommand)]
pub enum AttachmentAction {
    List {
        page_id: String,
        #[arg(long)]
        filename: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
    Upload {
        page_id: String,
        file: PathBuf,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long)]
        minor_edit: bool,
    },
}
