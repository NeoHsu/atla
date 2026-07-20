use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::AttachmentMode;

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
