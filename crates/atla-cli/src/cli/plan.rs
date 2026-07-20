use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::*;

#[derive(Debug, Subcommand)]
pub enum PlannableCommand {
    /// Plan a Jira mutation
    Jira(PlanJiraCommand),
    /// Plan a Confluence mutation
    Confluence(PlanConfluenceCommand),
}

impl PlannableCommand {
    pub fn into_command(self) -> Command {
        match self {
            Self::Jira(command) => Command::Jira(JiraCommand {
                resource: match command.resource {
                    PlanJiraResource::Issue(command) => JiraResource::Issue(IssueCommand {
                        action: match command.action {
                            PlanIssueAction::Create {
                                project,
                                issue_type,
                                summary,
                                description,
                                description_file,
                                fields,
                                labels,
                            } => IssueAction::Create {
                                project,
                                issue_type,
                                summary,
                                description,
                                description_file,
                                fields,
                                labels,
                            },
                        },
                    }),
                },
            }),
            Self::Confluence(command) => Command::Confluence(ConfluenceCommand {
                resource: match command.resource {
                    PlanConfluenceResource::Page(command) => {
                        ConfluenceResource::Page(PageCommand {
                            action: match command.action {
                                PlanPageAction::Create {
                                    space_id,
                                    title,
                                    parent,
                                    body,
                                    body_file,
                                    representation,
                                    numbered_table_rows,
                                    mentions,
                                    draft,
                                    private,
                                    root_level,
                                } => PageAction::Create {
                                    space: None,
                                    space_id: Some(space_id),
                                    title,
                                    parent,
                                    body,
                                    body_file,
                                    representation,
                                    numbered_table_rows,
                                    mentions,
                                    resolve_mentions: false,
                                    draft,
                                    private,
                                    root_level,
                                },
                                PlanPageAction::Update {
                                    id,
                                    title,
                                    parent,
                                    body,
                                    body_file,
                                    representation,
                                    numbered_table_rows,
                                    mentions,
                                    version,
                                    message,
                                    draft,
                                } => PageAction::Update {
                                    id,
                                    title,
                                    parent,
                                    body,
                                    body_file,
                                    representation,
                                    numbered_table_rows,
                                    mentions,
                                    resolve_mentions: false,
                                    version,
                                    message,
                                    draft,
                                },
                            },
                        })
                    }
                    PlanConfluenceResource::Blog(command) => {
                        ConfluenceResource::Blog(BlogCommand {
                            action: match command.action {
                                PlanBlogAction::Create {
                                    space_id,
                                    title,
                                    body,
                                    body_file,
                                    representation,
                                    draft,
                                    private,
                                } => BlogAction::Create {
                                    space: None,
                                    space_id: Some(space_id),
                                    title,
                                    body,
                                    body_file,
                                    representation,
                                    draft,
                                    private,
                                },
                                PlanBlogAction::Update {
                                    id,
                                    title,
                                    body,
                                    body_file,
                                    representation,
                                    version,
                                    message,
                                    draft,
                                } => BlogAction::Update {
                                    id,
                                    title: Some(title),
                                    body,
                                    body_file,
                                    representation,
                                    version: Some(version),
                                    message,
                                    draft,
                                },
                            },
                        })
                    }
                },
            }),
        }
    }
}

#[derive(Debug, Args)]
pub struct PlanJiraCommand {
    #[command(subcommand)]
    pub resource: PlanJiraResource,
}

#[derive(Debug, Subcommand)]
pub enum PlanJiraResource {
    /// Plan a Jira issue mutation
    Issue(PlanIssueCommand),
}

#[derive(Debug, Args)]
pub struct PlanIssueCommand {
    #[command(subcommand)]
    pub action: PlanIssueAction,
}

#[derive(Debug, Subcommand)]
pub enum PlanIssueAction {
    /// Plan creating an issue
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
}

#[derive(Debug, Args)]
pub struct PlanConfluenceCommand {
    #[command(subcommand)]
    pub resource: PlanConfluenceResource,
}

#[derive(Debug, Subcommand)]
pub enum PlanConfluenceResource {
    /// Plan a page mutation
    Page(PlanPageCommand),
    /// Plan a blog-post mutation
    Blog(PlanBlogCommand),
}

#[derive(Debug, Args)]
pub struct PlanPageCommand {
    #[command(subcommand)]
    pub action: PlanPageAction,
}

#[derive(Debug, Subcommand)]
pub enum PlanPageAction {
    /// Plan creating a page (requires an explicit space ID)
    Create {
        #[arg(long)]
        space_id: String,
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
        numbered_table_rows: bool,
        #[arg(long = "mention", value_name = "NAME=ACCOUNT_ID")]
        mentions: Vec<String>,
        #[arg(long)]
        draft: bool,
        #[arg(long)]
        private: bool,
        #[arg(long, conflicts_with = "parent")]
        root_level: bool,
    },
    /// Plan updating a page; full updates require explicit title, body, and version
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
        numbered_table_rows: bool,
        #[arg(long = "mention", value_name = "NAME=ACCOUNT_ID")]
        mentions: Vec<String>,
        #[arg(long)]
        version: Option<u64>,
        #[arg(long)]
        message: Option<String>,
        #[arg(long)]
        draft: bool,
    },
}

#[derive(Debug, Args)]
pub struct PlanBlogCommand {
    #[command(subcommand)]
    pub action: PlanBlogAction,
}

#[derive(Debug, Subcommand)]
pub enum PlanBlogAction {
    /// Plan creating a blog post (requires an explicit space ID)
    Create {
        #[arg(long)]
        space_id: String,
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
    /// Plan updating a blog post with explicit body, title, and version
    Update {
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        #[arg(long, required_unless_present = "body")]
        body_file: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = BodyRepresentation::Storage)]
        representation: BodyRepresentation,
        #[arg(long)]
        version: u64,
        #[arg(long)]
        message: Option<String>,
        #[arg(long)]
        draft: bool,
    },
}
