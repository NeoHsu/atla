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
    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
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
    /// Create or update a profile and store its API token
    Login {
        /// Site URL, e.g. https://your-site.atlassian.net
        #[arg(long)]
        instance: Option<String>,
        /// Atlassian account email
        #[arg(long)]
        email: Option<String>,
        /// API token (create at id.atlassian.com); omit to be prompted
        #[arg(long)]
        token: Option<String>,
        /// Token storage backend
        #[arg(long, value_enum)]
        storage: Option<AuthStorage>,
    },
    /// Remove stored credentials for the active profile
    Logout,
    /// Show authentication status for the active profile
    Status,
    /// Set the default profile
    Switch {
        /// Profile name to switch to
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
    /// Set a config key (run `atla config list` to see keys)
    Set { key: String, value: String },
    /// Print one config value
    Get { key: String },
    /// Print all config entries
    List,
}

#[derive(Debug, Args)]
pub struct JiraCommand {
    #[command(subcommand)]
    pub resource: JiraResource,
}

#[derive(Debug, Subcommand)]
pub enum JiraResource {
    /// Create, inspect, and update issues
    Issue(IssueCommand),
    /// List and inspect projects
    Project(ProjectCommand),
    /// Manage sprints on a board
    Sprint(SprintCommand),
    /// List and inspect boards
    Board(BoardCommand),
    /// Run a JQL query with automatic pagination
    #[command(after_help = "Examples:
  atla jira search 'assignee = currentUser() AND statusCategory != Done ORDER BY updated DESC'
  atla -o json jira search 'project = PROJ' --fields summary,status --limit 20")]
    Search {
        /// JQL query, e.g. 'project = PROJ AND statusCategory != Done'
        jql: String,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
        /// Comma-separated Jira fields to include, e.g. summary,status,assignee
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
    /// List issues by filters or raw JQL
    List {
        /// Filter by project key
        #[arg(long)]
        project: Option<String>,
        /// Filter by status name
        #[arg(long)]
        status: Option<String>,
        /// Filter by issue type name
        #[arg(long = "type")]
        issue_type: Option<String>,
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
        /// Raw JQL (overrides the other filters)
        #[arg(long)]
        jql: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
        /// Comma-separated Jira fields to include, e.g. summary,status,assignee
        #[arg(long)]
        fields: Option<String>,
    },
    /// Create an issue
    #[command(after_help = "Examples:
  atla jira issue create --project PROJ --type Task --summary 'Fix login'
  atla jira issue fields --project PROJ --type Bug --required-only   # discover required custom fields first
  atla jira issue create --project PROJ --type Bug --summary 'Crash' --field 'customfield_10166=\"5.1.0\"'")]
    Create {
        /// Project key
        #[arg(long)]
        project: String,
        /// Issue type name (see `jira project issue-types`)
        #[arg(long = "type")]
        issue_type: String,
        /// Issue summary
        #[arg(long)]
        summary: String,
        /// Description in Markdown (converted to ADF)
        #[arg(long, conflicts_with = "description_file", allow_hyphen_values = true)]
        description: Option<String>,
        /// Read the description from a Markdown file
        #[arg(long)]
        description_file: Option<PathBuf>,
        /// Set a field: ID=VALUE; string fields need JSON quotes: --field 'customfield_1="5.1"' (see `jira issue fields`)
        #[arg(long = "field")]
        fields: Vec<String>,
        /// Comma-separated labels
        #[arg(long)]
        labels: Option<String>,
    },
    /// Update summary, description, fields, or labels (alias: edit)
    #[command(alias = "edit")]
    Update {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// New summary
        #[arg(long)]
        summary: Option<String>,
        /// New description in Markdown
        #[arg(long, conflicts_with = "description_file", allow_hyphen_values = true)]
        description: Option<String>,
        /// Read the description from a Markdown file
        #[arg(long)]
        description_file: Option<PathBuf>,
        /// Set a field: ID=VALUE (see `jira issue fields`)
        #[arg(long = "field")]
        fields: Vec<String>,
        /// Comma-separated labels; supports add:/remove: prefixes
        #[arg(long)]
        labels: Option<String>,
    },
    /// Show one issue
    View {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Open in the browser instead of printing
        #[arg(long)]
        web: bool,
        /// Comma-separated Jira fields to include, e.g. summary,status,assignee
        #[arg(long)]
        fields: Option<String>,
        /// Also fetch GitHub pull requests and commits from the development panel
        #[arg(long)]
        with_github: bool,
    },
    /// Delete an issue (requires --yes)
    Delete {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Also delete subtasks
        #[arg(long)]
        delete_subtasks: bool,
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
    },
    /// Assign, reassign, or unassign an issue
    Assign {
        /// Issue key, e.g. PROJ-123
        key: String,
        #[command(flatten)]
        target: IssueAssignTargetArgs,
        /// Treat --to as an Atlassian account ID
        #[arg(long)]
        account_id: bool,
    },
    /// Move an issue through its workflow
    Transition {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Target status or transition name (omit to list options)
        #[arg(long)]
        to: Option<String>,
        /// Set a field during the transition: ID=VALUE
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    /// Manage issue comments
    Comment {
        #[command(subcommand)]
        action: IssueCommentAction,
    },
    /// Manage issue attachments
    Attachment {
        #[command(subcommand)]
        action: IssueAttachmentAction,
    },
    /// Manage issue links and GitHub dev-panel data
    Link {
        #[command(subcommand)]
        action: IssueLinkAction,
    },
    /// Log and list time spent
    Worklog {
        #[command(subcommand)]
        action: IssueWorklogAction,
    },
    /// List fields required to create an issue in a project
    Fields {
        /// Project key
        #[arg(long)]
        project: String,
        /// Issue type name
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
    /// Assignee: display name, email, or `me`
    #[arg(long)]
    pub to: Option<String>,
    /// Clear the assignee
    #[arg(long)]
    pub unassign: bool,
}

#[derive(Debug, Subcommand)]
pub enum IssueCommentAction {
    /// Add a comment
    #[command(after_help = "Examples:
  atla jira issue comment add PROJ-123 'Deployed to staging'
  atla jira issue comment add PROJ-123 --body-file notes.md --attachment error.log --attachment-mode link")]
    Add {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Comment body in Markdown
        #[arg(conflicts_with = "body_file", conflicts_with = "body_flag")]
        body: Option<String>,
        /// Comment body (alternative to the positional argument)
        #[arg(long = "body", conflicts_with = "body_file", conflicts_with = "body")]
        body_flag: Option<String>,
        /// Read the body from a file
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Upload files to the issue and reference them from the comment.
        #[arg(long = "attachment", value_name = "FILE")]
        attachments: Vec<PathBuf>,
        /// How to reference comment attachments.
        #[arg(long, value_enum, default_value_t = AttachmentMode::Auto)]
        attachment_mode: AttachmentMode,
    },
    /// List comments
    List {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Edit a comment
    Update {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Comment ID
        comment_id: String,
        /// New comment body in Markdown
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the body from a file
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
    /// Delete a comment (requires --yes)
    Delete {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Comment ID
        comment_id: String,
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum IssueAttachmentAction {
    /// Upload a file to an issue
    Upload {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// File to upload
        #[arg(long)]
        file: PathBuf,
    },
    /// List attachments on an issue
    List {
        /// Issue key, e.g. PROJ-123
        key: String,
    },
    /// Download one attachment by ID, or all with --all
    Download {
        /// Attachment ID, or an issue key with --all
        target: String,
        /// Download every attachment on the issue
        #[arg(long)]
        all: bool,
        /// Target file or directory
        #[arg(long = "dest")]
        dest: Option<PathBuf>,
    },
    /// Delete an attachment (requires --yes)
    Delete {
        /// Attachment ID
        attachment_id: String,
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum IssueLinkAction {
    /// Link this issue to another issue
    Add {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Link type name, e.g. Blocks or Relates
        #[arg(long = "type")]
        link_type: String,
        /// Issue key to link to
        #[arg(long)]
        target: String,
    },
    /// List linked issues
    List {
        /// Issue key, e.g. PROJ-123
        key: String,
    },
    /// Remove a link by its ID (see `link list`)
    Remove {
        /// Link ID (from `link list`)
        link_id: String,
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
    },
    /// List GitHub pull requests linked via the Jira development panel
    GithubLinks {
        /// Issue key, e.g. PROJ-123
        key: String,
    },
    /// List GitHub commits linked via the Jira development panel
    GithubCommits {
        /// Issue key, e.g. PROJ-123
        key: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum IssueWorklogAction {
    /// Log time spent on an issue
    Add {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Time spent, e.g. 45m or 2h
        #[arg(long)]
        time: String,
        /// Worklog comment
        #[arg(long)]
        comment: Option<String>,
        /// Start time (ISO-8601; default: now)
        #[arg(long)]
        started: Option<String>,
    },
    /// List worklogs
    List {
        /// Issue key, e.g. PROJ-123
        key: String,
        /// Max records to return; atla paginates the API internally to reach it
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
    /// List boards
    List {
        /// Filter by project key
        #[arg(long)]
        project: Option<String>,
        /// Board type: scrum or kanban
        #[arg(long = "type")]
        board_type: Option<String>,
        /// Filter by board name
        #[arg(long)]
        name: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Show one board
    View {
        /// Board ID
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
    /// List sprints for a board
    List {
        /// Board ID (see `jira board list`)
        #[arg(long)]
        board: u64,
        /// Filter by state: future, active, or closed
        #[arg(long)]
        state: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Show the active sprints for a board
    Active {
        /// Board ID (see `jira board list`)
        #[arg(long)]
        board: u64,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Show one sprint
    View {
        /// Sprint ID
        id: u64,
    },
    /// Create a sprint
    Create {
        /// Board ID (see `jira board list`)
        #[arg(long)]
        board: u64,
        /// Sprint name
        #[arg(long)]
        name: String,
        /// Start date (ISO-8601)
        #[arg(long)]
        start: Option<String>,
        /// End date (ISO-8601)
        #[arg(long)]
        end: Option<String>,
        /// Sprint goal
        #[arg(long)]
        goal: Option<String>,
    },
    /// Start a sprint
    Start {
        /// Sprint ID
        id: u64,
        /// Start date (ISO-8601; default: now)
        #[arg(long)]
        start: Option<String>,
        /// End date (ISO-8601)
        #[arg(long)]
        end: Option<String>,
    },
    /// Close a sprint
    Close {
        /// Sprint ID
        id: u64,
    },
    /// Move issues into the sprint
    Add {
        /// Sprint ID
        id: u64,
        /// Issue keys (comma-separated or repeated)
        #[arg(long, alias = "issue", value_delimiter = ',')]
        issues: Vec<String>,
    },
    /// Move issues back to the backlog
    Remove {
        /// Sprint ID
        id: u64,
        /// Issue keys (comma-separated or repeated)
        #[arg(long, alias = "issue", value_delimiter = ',')]
        issues: Vec<String>,
    },
    /// List issues in the sprint
    Issues {
        /// Sprint ID
        id: u64,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
        /// Comma-separated Jira fields to include, e.g. summary,status,assignee
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
    /// List projects
    List {
        /// Match project name or key
        #[arg(long)]
        query: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Show one project
    View {
        /// Project key
        key: String,
    },
    /// List issue types valid for `issue create --type`
    IssueTypes {
        /// Project key
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
    /// Create, read, and update pages
    Page(PageCommand),
    /// Manage spaces
    Space(SpaceCommand),
    /// Manage blog posts
    Blog(BlogCommand),
    /// Run a CQL query with automatic pagination
    Search {
        /// CQL query, e.g. 'type = page AND space = ENG'
        cql: String,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Manage page attachments
    Attachment(AttachmentCommand),
}

#[derive(Debug, Args)]
pub struct PageCommand {
    #[command(subcommand)]
    pub action: PageAction,
}

#[derive(Debug, Subcommand)]
pub enum PageAction {
    /// Create a page
    #[command(after_help = "Examples:
  atla confluence page create --space ENG --title 'Notes' --body-file notes.md --representation markdown
  atla confluence page create --space ENG --title 'Raw' --body '<p>storage XHTML</p>'   # default representation is storage")]
    Create {
        /// Space key, e.g. ENG
        #[arg(short = 's', long)]
        space: Option<String>,
        /// Numeric space ID (alternative to --space)
        #[arg(long)]
        space_id: Option<String>,
        /// Page title
        #[arg(long)]
        title: String,
        /// Parent page ID
        #[arg(long, conflicts_with = "root_level")]
        parent: Option<String>,
        /// Body content inline
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the body from a file
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// How to interpret the body; use `markdown` for Markdown input
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
        /// Operate on a draft (unpublished) version
        #[arg(long)]
        draft: bool,
        /// Restrict visibility to yourself
        #[arg(long)]
        private: bool,
        /// Create at the space root instead of under a parent
        #[arg(long, conflicts_with = "parent")]
        root_level: bool,
    },
    /// List pages
    List {
        /// Space key, e.g. ENG
        #[arg(short = 's', long)]
        space: Option<String>,
        /// Numeric space ID (alternative to --space)
        #[arg(long)]
        space_id: Option<String>,
        /// Filter by title
        #[arg(long)]
        title: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Show page metadata; pass --format to print the body
    #[command(
        after_help = "Without --format only metadata is printed. To read the content:
  atla confluence page view 123456 --format markdown"
    )]
    View {
        /// Page ID
        id: String,
        /// Open in the browser instead of printing
        #[arg(long)]
        web: bool,
        /// Print the body in this format
        #[arg(long, value_enum)]
        format: Option<ContentViewFormat>,
        /// Emit atla Markdown directives for ADF table metadata (requires --format markdown).
        #[arg(long)]
        preserve_table_options: bool,
        /// Also list the page's attachments
        #[arg(long)]
        with_attachments: bool,
    },
    /// List child pages (--depth for deeper descendants)
    Children {
        /// Page ID
        id: String,
        /// Descend this many levels (default: 1)
        #[arg(long)]
        depth: Option<u32>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Copy a page into another location
    Copy {
        /// Page ID to copy
        source_id: String,
        /// Title for the copy
        #[arg(long)]
        title: String,
        /// Space key, e.g. ENG
        #[arg(short = 's', long)]
        space: Option<String>,
        /// Numeric space ID (alternative to --space)
        #[arg(long)]
        space_id: Option<String>,
        /// Parent page ID for the copy
        #[arg(long)]
        parent: Option<String>,
        /// Place at the space root instead of under a parent
        #[arg(long)]
        root_level: bool,
    },
    /// Update title, body, or location
    Update {
        /// Page ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New parent page ID
        #[arg(long)]
        parent: Option<String>,
        /// Body content inline
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the body from a file
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// How to interpret the body; use `markdown` for Markdown input
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
        /// Explicit next version number (default: current + 1)
        #[arg(long)]
        version: Option<u64>,
        /// Version comment recorded in page history
        #[arg(long)]
        message: Option<String>,
        /// Operate on a draft (unpublished) version
        #[arg(long)]
        draft: bool,
    },
    /// Delete a page (requires --yes)
    Delete {
        /// Page ID
        id: String,
        /// Permanently purge instead of moving to trash
        #[arg(long)]
        purge: bool,
        /// Operate on a draft (unpublished) version
        #[arg(long)]
        draft: bool,
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
    },
    /// Move a page under a new parent
    Move {
        /// Page ID
        id: String,
        /// New parent page ID
        #[arg(long)]
        parent: String,
    },
    /// Manage page labels
    Label {
        #[command(subcommand)]
        action: PageLabelAction,
    },
    /// Manage page comments
    Comment {
        #[command(subcommand)]
        action: PageCommentAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum PageLabelAction {
    /// List labels on a page
    List {
        /// Page ID
        page_id: String,
        /// Filter labels by prefix
        #[arg(long)]
        prefix: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Add labels to a page
    Add {
        /// Page ID
        page_id: String,
        /// Labels to add
        labels: Vec<String>,
    },
    /// Remove a label from a page
    Remove {
        /// Page ID
        page_id: String,
        /// Label to remove
        label: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PageCommentAction {
    /// List comments on a page
    List {
        /// Page ID
        page_id: String,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Add a comment to a page
    Add {
        /// Page ID
        page_id: String,
        /// Comment body (interpretation follows --representation)
        #[arg(conflicts_with = "body_file", conflicts_with = "body_flag")]
        body: Option<String>,
        /// Comment body (alternative to the positional argument)
        #[arg(long = "body", conflicts_with = "body_file", conflicts_with = "body")]
        body_flag: Option<String>,
        /// Read the body from a file
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Parent comment ID (reply)
        #[arg(long)]
        parent: Option<String>,
        /// How to interpret the body; use `markdown` for Markdown input
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
    /// Delete a page comment (requires --yes)
    Delete {
        /// Page ID
        page_id: String,
        /// Comment ID
        comment_id: String,
        /// Confirm the destructive operation (required; there is no prompt)
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
    /// Create a blog post
    Create {
        /// Space key, e.g. ENG
        #[arg(short = 's', long)]
        space: Option<String>,
        /// Numeric space ID (alternative to --space)
        #[arg(long)]
        space_id: Option<String>,
        /// Blog post title
        #[arg(long)]
        title: String,
        /// Body content inline
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the body from a file
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// How to interpret the body; use `markdown` for Markdown input
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
        /// Operate on a draft (unpublished) version
        #[arg(long)]
        draft: bool,
        /// Restrict visibility to yourself
        #[arg(long)]
        private: bool,
    },
    /// List blog posts
    List {
        /// Space key, e.g. ENG
        #[arg(short = 's', long)]
        space: Option<String>,
        /// Numeric space ID (alternative to --space)
        #[arg(long)]
        space_id: Option<String>,
        /// Filter by title
        #[arg(long)]
        title: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Show blog metadata; pass --format to print the body
    View {
        /// Blog post ID
        id: String,
        /// Print the body in this format
        #[arg(long, value_enum)]
        format: Option<ContentViewFormat>,
    },
    /// Update a blog post
    Update {
        /// Blog post ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// Body content inline
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the body from a file
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// How to interpret the body; use `markdown` for Markdown input
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
        /// Explicit next version number (default: current + 1)
        #[arg(long)]
        version: Option<u64>,
        /// Version comment
        #[arg(long)]
        message: Option<String>,
        /// Operate on a draft (unpublished) version
        #[arg(long)]
        draft: bool,
    },
    /// Delete a blog post (requires --yes)
    Delete {
        /// Blog post ID
        id: String,
        /// Permanently purge instead of moving to trash
        #[arg(long)]
        purge: bool,
        /// Operate on a draft (unpublished) version
        #[arg(long)]
        draft: bool,
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
    },
    /// Manage blog labels
    Label {
        #[command(subcommand)]
        action: BlogLabelAction,
    },
    /// Manage blog comments
    Comment {
        #[command(subcommand)]
        action: BlogCommentAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum BlogLabelAction {
    /// List labels on a blog post
    List {
        /// Blog post ID
        blog_id: String,
        /// Filter labels by prefix
        #[arg(long)]
        prefix: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Add labels to a blog post
    Add {
        /// Blog post ID
        blog_id: String,
        /// Labels to add
        labels: Vec<String>,
    },
    /// Remove a label from a blog post
    Remove {
        /// Blog post ID
        blog_id: String,
        /// Label to remove
        label: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BlogCommentAction {
    /// List comments on a blog post
    List {
        /// Blog post ID
        blog_id: String,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Add a comment to a blog post
    Add {
        /// Blog post ID
        blog_id: String,
        /// Comment body (interpretation follows --representation)
        #[arg(conflicts_with = "body_file", conflicts_with = "body_flag")]
        body: Option<String>,
        /// Comment body (alternative to the positional argument)
        #[arg(long = "body", conflicts_with = "body_file", conflicts_with = "body")]
        body_flag: Option<String>,
        /// Read the body from a file
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Parent comment ID (reply)
        #[arg(long)]
        parent: Option<String>,
        /// How to interpret the body; use `markdown` for Markdown input
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
    },
    /// Delete a blog comment (requires --yes)
    Delete {
        /// Blog post ID
        blog_id: String,
        /// Comment ID
        comment_id: String,
        /// Confirm the destructive operation (required; there is no prompt)
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
    /// List spaces
    List {
        /// Filter by space key
        #[arg(long)]
        key: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Show one space
    View {
        /// Space key, e.g. ENG
        key: String,
    },
    /// Create a space (requires --key or --alias)
    Create {
        /// Space name
        name: String,
        /// Space key (required unless --alias is given)
        #[arg(long)]
        key: Option<String>,
        /// Space alias (alternative to --key)
        #[arg(long)]
        alias: Option<String>,
        /// Space description
        #[arg(long, conflicts_with = "description_file", allow_hyphen_values = true)]
        description: Option<String>,
        #[arg(long, conflicts_with = "description")]
        description_file: Option<PathBuf>,
        /// Restrict visibility to yourself
        #[arg(long)]
        private: bool,
    },
    /// Update space name or description
    Update {
        /// Space key, e.g. ENG
        key: String,
        /// New space name
        #[arg(long)]
        name: Option<String>,
        /// New space description
        #[arg(long, conflicts_with = "description_file", allow_hyphen_values = true)]
        description: Option<String>,
        #[arg(long, conflicts_with = "description")]
        description_file: Option<PathBuf>,
    },
    /// Delete a space (requires --yes)
    Delete {
        /// Space key, e.g. ENG
        key: String,
        /// Confirm the destructive operation (required; there is no prompt)
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
    /// List attachments on a page
    List {
        /// Page ID
        page_id: String,
        /// Filter by filename
        #[arg(long)]
        filename: Option<String>,
        /// Max records to return; atla paginates the API internally to reach it
        #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// Fetch every matching record (overrides --limit; suppresses truncation warning)
        #[arg(long, conflicts_with = "limit", conflicts_with = "page_token")]
        all: bool,
        /// Continue from a token printed by a previous page
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Show attachment metadata
    View {
        /// Attachment ID (with or without the `att` prefix)
        attachment_id: String,
    },
    /// Upload a file to a page
    Upload {
        /// Page ID
        page_id: String,
        /// File to upload
        file: PathBuf,
        /// Attachment comment
        #[arg(long)]
        comment: Option<String>,
        /// Do not notify page watchers
        #[arg(long)]
        minor_edit: bool,
    },
    /// Download an attachment
    Download {
        /// Attachment ID (with or without the `att` prefix)
        attachment_id: String,
        /// Write to FILE (default: attachment filename in the current directory)
        #[arg(long = "save-to", short = 'f', value_name = "FILE")]
        save_to: Option<PathBuf>,
    },
    /// Delete an attachment (requires --yes)
    Delete {
        /// Attachment ID (with or without the `att` prefix)
        attachment_id: String,
        /// Permanently purge instead of moving to trash
        #[arg(long)]
        purge: bool,
        /// Confirm the destructive operation (required; there is no prompt)
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
