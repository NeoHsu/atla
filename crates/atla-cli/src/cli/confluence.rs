use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::{AttachmentMode, BodyRepresentation, ContentViewFormat};

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
        #[arg(
            long,
            conflicts_with_all = [
                "format",
                "metadata_only",
                "fields",
                "max_chars",
                "preserve_table_options",
                "with_attachments"
            ]
        )]
        web: bool,
        /// Print the body in this format
        #[arg(long, value_enum)]
        format: Option<ContentViewFormat>,
        /// Explicitly return metadata without fetching the body (the default in 0.6)
        #[arg(long, conflicts_with = "format")]
        metadata_only: bool,
        /// Select top-level JSON fields (comma-separated; requires --output json)
        #[arg(long, value_name = "FIELD,...")]
        fields: Option<String>,
        /// Truncate the rendered body after this many Unicode characters
        #[arg(long, requires = "format", value_parser = clap::value_parser!(u32).range(1..))]
        max_chars: Option<u32>,
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
        /// Permanently remove an already-trashed page (requires space admin)
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
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
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
    #[command(
        after_help = "Exactly one comment body source is required: positional BODY, --body, or --body-file."
    )]
    Add {
        /// Page ID
        page_id: String,
        /// Comment body (interpretation follows --representation)
        #[arg(
            conflicts_with = "body_file",
            conflicts_with = "body_flag",
            required_unless_present_any = ["body_flag", "body_file"]
        )]
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
    #[command(
        after_help = "Without --format only metadata is printed. To read the content:\n  atla confluence blog view 123456 --format markdown"
    )]
    View {
        /// Blog post ID
        id: String,
        /// Print the body in this format
        #[arg(long, value_enum)]
        format: Option<ContentViewFormat>,
        /// Explicitly return metadata without fetching the body (the default in 0.6)
        #[arg(long, conflicts_with = "format")]
        metadata_only: bool,
        /// Select top-level JSON fields (comma-separated; requires --output json)
        #[arg(long, value_name = "FIELD,...")]
        fields: Option<String>,
        /// Truncate the rendered body after this many Unicode characters
        #[arg(long, requires = "format", value_parser = clap::value_parser!(u32).range(1..))]
        max_chars: Option<u32>,
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
        /// Permanently remove an already-trashed blog post (requires space admin)
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
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
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
    #[command(
        after_help = "Exactly one comment body source is required: positional BODY, --body, or --body-file."
    )]
    Add {
        /// Blog post ID
        blog_id: String,
        /// Comment body (interpretation follows --representation)
        #[arg(
            conflicts_with = "body_file",
            conflicts_with = "body_flag",
            required_unless_present_any = ["body_flag", "body_file"]
        )]
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
        #[arg(long, conflicts_with = "alias", required_unless_present = "alias")]
        key: Option<String>,
        /// Space alias (alternative to --key)
        #[arg(long, conflicts_with = "key")]
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
    #[command(
        after_help = "At least one of --name, --description, or --description-file is required."
    )]
    Update {
        /// Space key, e.g. ENG
        key: String,
        /// New space name
        #[arg(
            long,
            required_unless_present_any = ["description", "description_file"]
        )]
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
        /// Permanently remove an already-trashed attachment (requires space admin)
        #[arg(long)]
        purge: bool,
        /// Confirm the destructive operation (required; there is no prompt)
        #[arg(long)]
        yes: bool,
    },
}
