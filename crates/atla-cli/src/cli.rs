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
    Auth(AuthCommand),
    Config(ConfigCommand),
    Jira(JiraCommand),
    Confluence(ConfluenceCommand),
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
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
        #[arg(long, value_enum)]
        storage: Option<AuthStorage>,
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
    Sprint(SprintCommand),
    Board(BoardCommand),
    Search {
        jql: String,
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
        #[arg(long)]
        fields: Option<String>,
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
        #[arg(long = "type")]
        issue_type: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        jql: Option<String>,
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
        #[arg(long)]
        fields: Option<String>,
    },
    Create {
        #[arg(long)]
        project: String,
        #[arg(long = "type")]
        issue_type: String,
        #[arg(long)]
        summary: String,
        #[arg(long, conflicts_with = "description_file", allow_hyphen_values = true)]
        description: Option<String>,
        #[arg(long)]
        description_file: Option<PathBuf>,
        #[arg(long = "field")]
        fields: Vec<String>,
        #[arg(long)]
        labels: Option<String>,
    },
    #[command(alias = "edit")]
    Update {
        key: String,
        #[arg(long)]
        summary: Option<String>,
        #[arg(long, conflicts_with = "description_file", allow_hyphen_values = true)]
        description: Option<String>,
        #[arg(long)]
        description_file: Option<PathBuf>,
        #[arg(long = "field")]
        fields: Vec<String>,
        #[arg(long)]
        labels: Option<String>,
    },
    View {
        key: String,
        #[arg(long)]
        web: bool,
        #[arg(long)]
        fields: Option<String>,
        /// Also fetch GitHub pull requests and commits from the development panel
        #[arg(long)]
        with_github: bool,
    },
    Delete {
        key: String,
        #[arg(long)]
        delete_subtasks: bool,
        #[arg(long)]
        yes: bool,
    },
    Assign {
        key: String,
        #[command(flatten)]
        target: IssueAssignTargetArgs,
        #[arg(long)]
        account_id: bool,
    },
    Transition {
        key: String,
        #[arg(long)]
        to: Option<String>,
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    Comment {
        #[command(subcommand)]
        action: IssueCommentAction,
    },
    Attachment {
        #[command(subcommand)]
        action: IssueAttachmentAction,
    },
    Link {
        #[command(subcommand)]
        action: IssueLinkAction,
    },
    Worklog {
        #[command(subcommand)]
        action: IssueWorklogAction,
    },
    /// List fields required to create an issue in a project
    Fields {
        #[arg(long)]
        project: String,
        #[arg(long = "type")]
        issue_type: String,
        /// Show only required fields (default: show all)
        #[arg(long)]
        required_only: bool,
    },
}

#[derive(Debug, Args)]
#[group(id = "assign_target", required = true, multiple = false)]
pub struct IssueAssignTargetArgs {
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub unassign: bool,
}

#[derive(Debug, Subcommand)]
pub enum IssueCommentAction {
    Add {
        key: String,
        #[arg(conflicts_with = "body_file", conflicts_with = "body_flag")]
        body: Option<String>,
        #[arg(long = "body", conflicts_with = "body_file", conflicts_with = "body")]
        body_flag: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Upload files to the issue and reference them from the comment.
        #[arg(long = "attachment", value_name = "FILE")]
        attachments: Vec<PathBuf>,
        /// How to reference comment attachments.
        #[arg(long, value_enum, default_value_t = AttachmentMode::Auto)]
        attachment_mode: AttachmentMode,
    },
    List {
        key: String,
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    Update {
        key: String,
        comment_id: String,
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
    Delete {
        key: String,
        comment_id: String,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum IssueAttachmentAction {
    Upload {
        key: String,
        #[arg(long)]
        file: PathBuf,
    },
    List {
        key: String,
    },
    Download {
        target: String,
        #[arg(long)]
        all: bool,
        #[arg(long = "dest")]
        dest: Option<PathBuf>,
    },
    Delete {
        attachment_id: String,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum IssueLinkAction {
    Add {
        key: String,
        #[arg(long = "type")]
        link_type: String,
        #[arg(long)]
        target: String,
    },
    List {
        key: String,
    },
    Remove {
        link_id: String,
        #[arg(long)]
        yes: bool,
    },
    /// List GitHub pull requests linked via the Jira development panel
    GithubLinks {
        key: String,
    },
    /// List GitHub commits linked via the Jira development panel
    GithubCommits {
        key: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum IssueWorklogAction {
    Add {
        key: String,
        #[arg(long)]
        time: String,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long)]
        started: Option<String>,
    },
    List {
        key: String,
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
}

#[derive(Debug, Args)]
pub struct BoardCommand {
    #[command(subcommand)]
    pub action: BoardAction,
}

#[derive(Debug, Subcommand)]
pub enum BoardAction {
    List {
        #[arg(long)]
        project: Option<String>,
        #[arg(long = "type")]
        board_type: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    View {
        id: u64,
    },
}

#[derive(Debug, Args)]
pub struct SprintCommand {
    #[command(subcommand)]
    pub action: SprintAction,
}

#[derive(Debug, Subcommand)]
pub enum SprintAction {
    List {
        #[arg(long)]
        board: u64,
        #[arg(long)]
        state: Option<String>,
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    Active {
        #[arg(long)]
        board: u64,
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    View {
        id: u64,
    },
    Create {
        #[arg(long)]
        board: u64,
        #[arg(long)]
        name: String,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
        #[arg(long)]
        goal: Option<String>,
    },
    Start {
        id: u64,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
    },
    Close {
        id: u64,
    },
    Add {
        id: u64,
        #[arg(long, alias = "issue", value_delimiter = ',')]
        issues: Vec<String>,
    },
    Remove {
        id: u64,
        #[arg(long, alias = "issue", value_delimiter = ',')]
        issues: Vec<String>,
    },
    Issues {
        id: u64,
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
        #[arg(long)]
        fields: Option<String>,
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
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    View {
        key: String,
    },
    IssueTypes {
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
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
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
        /// Enable Confluence numbered rows for Markdown tables (requires --representation markdown).
        #[arg(long)]
        numbered_table_rows: bool,
        /// Convert mapped Markdown mentions to ADF mentions; pass NAME=ACCOUNT_ID.
        #[arg(long = "mention", value_name = "NAME=ACCOUNT_ID")]
        mentions: Vec<String>,
        /// Resolve Markdown mentions by searching Atlassian users (requires --representation markdown).
        #[arg(long)]
        resolve_mentions: bool,
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
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    View {
        id: String,
        #[arg(long)]
        web: bool,
        #[arg(long, value_enum)]
        format: Option<ContentViewFormat>,
        /// Emit atla Markdown directives for ADF table metadata (requires --format markdown).
        #[arg(long)]
        preserve_table_options: bool,
        #[arg(long)]
        with_attachments: bool,
    },
    Children {
        id: String,
        #[arg(long)]
        depth: Option<u32>,
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    Copy {
        source_id: String,
        #[arg(long)]
        title: String,
        #[arg(short = 's', long)]
        space: Option<String>,
        #[arg(long)]
        space_id: Option<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        root_level: bool,
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
        /// Enable Confluence numbered rows for Markdown tables (requires --representation markdown).
        #[arg(long)]
        numbered_table_rows: bool,
        /// Convert mapped Markdown mentions to ADF mentions; pass NAME=ACCOUNT_ID.
        #[arg(long = "mention", value_name = "NAME=ACCOUNT_ID")]
        mentions: Vec<String>,
        /// Resolve Markdown mentions by searching Atlassian users (requires --representation markdown).
        #[arg(long)]
        resolve_mentions: bool,
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
    Label {
        #[command(subcommand)]
        action: PageLabelAction,
    },
    Comment {
        #[command(subcommand)]
        action: PageCommentAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum PageLabelAction {
    List {
        page_id: String,
        #[arg(long)]
        prefix: Option<String>,
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    Add {
        page_id: String,
        labels: Vec<String>,
    },
    Remove {
        page_id: String,
        label: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PageCommentAction {
    List {
        page_id: String,
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    Add {
        page_id: String,
        #[arg(conflicts_with = "body_file")]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
        /// Enable Confluence numbered rows for Markdown tables (requires --representation markdown).
        #[arg(long)]
        numbered_table_rows: bool,
        /// Convert mapped Markdown mentions to ADF mentions; pass NAME=ACCOUNT_ID.
        #[arg(long = "mention", value_name = "NAME=ACCOUNT_ID")]
        mentions: Vec<String>,
        /// Resolve Markdown mentions by searching Atlassian users (requires --representation markdown).
        #[arg(long)]
        resolve_mentions: bool,
        /// Upload files to the page and reference them from the comment.
        #[arg(long = "attachment", value_name = "FILE")]
        attachments: Vec<PathBuf>,
        /// How to reference comment attachments.
        #[arg(long, value_enum, default_value_t = AttachmentMode::Auto)]
        attachment_mode: AttachmentMode,
    },
    Delete {
        page_id: String,
        comment_id: String,
        #[arg(long)]
        yes: bool,
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
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    View {
        id: String,
        #[arg(long, value_enum)]
        format: Option<ContentViewFormat>,
    },
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
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
    Label {
        #[command(subcommand)]
        action: BlogLabelAction,
    },
    Comment {
        #[command(subcommand)]
        action: BlogCommentAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum BlogLabelAction {
    List {
        blog_id: String,
        #[arg(long)]
        prefix: Option<String>,
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    Add {
        blog_id: String,
        labels: Vec<String>,
    },
    Remove {
        blog_id: String,
        label: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BlogCommentAction {
    List {
        blog_id: String,
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    Add {
        blog_id: String,
        #[arg(conflicts_with = "body_file")]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
    },
    Delete {
        blog_id: String,
        comment_id: String,
        #[arg(long)]
        yes: bool,
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
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    View {
        key: String,
    },
    Create {
        name: String,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        alias: Option<String>,
        #[arg(long, conflicts_with = "description_file", allow_hyphen_values = true)]
        description: Option<String>,
        #[arg(long, conflicts_with = "description")]
        description_file: Option<PathBuf>,
        #[arg(long)]
        private: bool,
    },
    Update {
        key: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, conflicts_with = "description_file", allow_hyphen_values = true)]
        description: Option<String>,
        #[arg(long, conflicts_with = "description")]
        description_file: Option<PathBuf>,
    },
    Delete {
        key: String,
        #[arg(long)]
        yes: bool,
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
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    View {
        attachment_id: String,
    },
    Upload {
        page_id: String,
        file: PathBuf,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long)]
        minor_edit: bool,
    },
    Download {
        attachment_id: String,
        #[arg(long = "save-to", short = 'f', value_name = "FILE")]
        save_to: Option<PathBuf>,
    },
    Delete {
        attachment_id: String,
        #[arg(long)]
        purge: bool,
        #[arg(long)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn page_create_accepts_markdown_mention_options() {
        let cli = Cli::try_parse_from([
            "atla",
            "confluence",
            "page",
            "create",
            "--space",
            "ENG",
            "--title",
            "Runbook",
            "--body",
            "@Neo please review",
            "--representation",
            "markdown",
            "--mention",
            "Neo=account-neo",
            "--resolve-mentions",
        ])
        .expect("parse cli");

        let Command::Confluence(command) = cli.command else {
            panic!("expected confluence command");
        };
        let ConfluenceResource::Page(command) = command.resource else {
            panic!("expected page command");
        };
        let PageAction::Create {
            mentions,
            resolve_mentions,
            ..
        } = command.action
        else {
            panic!("expected page create action");
        };
        assert_eq!(mentions, vec!["Neo=account-neo"]);
        assert!(resolve_mentions);
    }

    #[test]
    fn page_view_accepts_preserve_table_options_flag() {
        let cli = Cli::try_parse_from([
            "atla",
            "confluence",
            "page",
            "view",
            "123456",
            "--format",
            "markdown",
            "--preserve-table-options",
        ])
        .expect("parse cli");

        let Command::Confluence(command) = cli.command else {
            panic!("expected confluence command");
        };
        let ConfluenceResource::Page(command) = command.resource else {
            panic!("expected page command");
        };
        let PageAction::View {
            id,
            format,
            preserve_table_options,
            ..
        } = command.action
        else {
            panic!("expected page view action");
        };
        assert_eq!(id, "123456");
        assert!(matches!(format, Some(ContentViewFormat::Markdown)));
        assert!(preserve_table_options);
    }

    #[test]
    fn jira_comment_add_accepts_attachment_options() {
        let cli = Cli::try_parse_from([
            "atla",
            "jira",
            "issue",
            "comment",
            "add",
            "PROJ-123",
            "please check",
            "--attachment",
            "error.log",
            "--attachment-mode",
            "link",
        ])
        .expect("parse cli");

        let Command::Jira(command) = cli.command else {
            panic!("expected jira command");
        };
        let JiraResource::Issue(command) = command.resource else {
            panic!("expected issue command");
        };
        let IssueAction::Comment { action } = command.action else {
            panic!("expected comment action");
        };
        let IssueCommentAction::Add {
            attachments,
            attachment_mode,
            ..
        } = action
        else {
            panic!("expected comment add action");
        };
        assert_eq!(attachments, vec![PathBuf::from("error.log")]);
        assert_eq!(attachment_mode, AttachmentMode::Link);
    }

    #[test]
    fn page_comment_add_accepts_attachment_options() {
        let cli = Cli::try_parse_from([
            "atla",
            "confluence",
            "page",
            "comment",
            "add",
            "123456",
            "please check",
            "--attachment",
            "report.pdf",
            "--attachment-mode",
            "embed",
        ])
        .expect("parse cli");

        let Command::Confluence(command) = cli.command else {
            panic!("expected confluence command");
        };
        let ConfluenceResource::Page(command) = command.resource else {
            panic!("expected page command");
        };
        let PageAction::Comment { action } = command.action else {
            panic!("expected page comment action");
        };
        let PageCommentAction::Add {
            attachments,
            attachment_mode,
            ..
        } = action
        else {
            panic!("expected page comment add action");
        };
        assert_eq!(attachments, vec![PathBuf::from("report.pdf")]);
        assert_eq!(attachment_mode, AttachmentMode::Embed);
    }

    #[test]
    fn attachment_download_accepts_save_to_flag() {
        let cli = Cli::try_parse_from([
            "atla",
            "-o",
            "json",
            "confluence",
            "attachment",
            "download",
            "att123",
            "--save-to",
            "download.txt",
        ])
        .expect("parse cli");

        assert_eq!(cli.global.output, Some(OutputFormat::Json));
        let Command::Confluence(command) = cli.command else {
            panic!("expected confluence command");
        };
        let ConfluenceResource::Attachment(command) = command.resource else {
            panic!("expected attachment command");
        };
        let AttachmentAction::Download {
            attachment_id,
            save_to,
        } = command.action
        else {
            panic!("expected download action");
        };
        assert_eq!(attachment_id, "att123");
        assert_eq!(
            save_to.as_deref(),
            Some(std::path::Path::new("download.txt"))
        );
    }

    #[test]
    fn attachment_download_accepts_short_file_flag() {
        let cli = Cli::try_parse_from([
            "atla",
            "confluence",
            "attachment",
            "download",
            "att123",
            "-f",
            "download.txt",
        ])
        .expect("parse cli");

        let Command::Confluence(command) = cli.command else {
            panic!("expected confluence command");
        };
        let ConfluenceResource::Attachment(command) = command.resource else {
            panic!("expected attachment command");
        };
        let AttachmentAction::Download { save_to, .. } = command.action else {
            panic!("expected download action");
        };
        assert_eq!(
            save_to.as_deref(),
            Some(std::path::Path::new("download.txt"))
        );
    }
}
