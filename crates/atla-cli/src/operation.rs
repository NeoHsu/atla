//! Central metadata registry for every CLI operation.
//!
//! Safety policy is derived from the parsed clap command rather than command
//! name heuristics, so aliases and argument values cannot bypass it.

use crate::cli::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationRisk {
    Read,
    Write,
    Destructive,
}

impl OperationRisk {
    pub fn mutates(self) -> bool {
        matches!(self, Self::Write | Self::Destructive)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationMetadata {
    pub id: &'static str,
    pub method: Option<&'static str>,
    pub risk: OperationRisk,
    pub paginated: bool,
    pub dry_run: bool,
}

impl OperationMetadata {
    pub fn is_retry_safe(self) -> bool {
        matches!(
            self.method,
            Some("GET" | "HEAD" | "PUT" | "DELETE" | "OPTIONS" | "TRACE")
        )
    }
}

const fn read(
    id: &'static str,
    method: Option<&'static str>,
    paginated: bool,
) -> OperationMetadata {
    OperationMetadata {
        id,
        method,
        risk: OperationRisk::Read,
        paginated,
        dry_run: true,
    }
}

const fn write(id: &'static str, method: Option<&'static str>) -> OperationMetadata {
    OperationMetadata {
        id,
        method,
        risk: OperationRisk::Write,
        paginated: false,
        dry_run: true,
    }
}

const fn destructive(id: &'static str, method: &'static str) -> OperationMetadata {
    OperationMetadata {
        id,
        method: Some(method),
        risk: OperationRisk::Destructive,
        paginated: false,
        dry_run: true,
    }
}

/// A bounded `--all` request behaves like a normal page so the caller gets a
/// resume token when a context budget stops collection.
pub fn apply_context_budgets(command: &mut Command, bounded: bool) {
    if !bounded {
        return;
    }
    match command {
        Command::Jira(command) => match &mut command.resource {
            JiraResource::Search { all, .. } => *all = false,
            JiraResource::Issue(command) => match &mut command.action {
                IssueAction::List { all, .. } => *all = false,
                IssueAction::Comment {
                    action: IssueCommentAction::List { all, .. },
                } => *all = false,
                _ => {}
            },
            JiraResource::Project(command) => {
                if let ProjectAction::List { all, .. } = &mut command.action {
                    *all = false;
                }
            }
            JiraResource::Sprint(command) => match &mut command.action {
                SprintAction::List { all, .. }
                | SprintAction::Active { all, .. }
                | SprintAction::Issues { all, .. } => *all = false,
                _ => {}
            },
            JiraResource::Board(command) => {
                if let BoardAction::List { all, .. } = &mut command.action {
                    *all = false;
                }
            }
        },
        Command::Confluence(command) => match &mut command.resource {
            ConfluenceResource::Search { all, .. } => *all = false,
            ConfluenceResource::Page(command) => match &mut command.action {
                PageAction::List { all, .. } | PageAction::Children { all, .. } => *all = false,
                PageAction::Label {
                    action: PageLabelAction::List { all, .. },
                }
                | PageAction::Comment {
                    action: PageCommentAction::List { all, .. },
                } => *all = false,
                _ => {}
            },
            ConfluenceResource::Space(command) => {
                if let SpaceAction::List { all, .. } = &mut command.action {
                    *all = false;
                }
            }
            ConfluenceResource::Blog(command) => match &mut command.action {
                BlogAction::List { all, .. } => *all = false,
                BlogAction::Label {
                    action: BlogLabelAction::List { all, .. },
                }
                | BlogAction::Comment {
                    action: BlogCommentAction::List { all, .. },
                } => *all = false,
                _ => {}
            },
            ConfluenceResource::Attachment(command) => {
                if let AttachmentAction::List { all, .. } = &mut command.action {
                    *all = false;
                }
            }
        },
        Command::Auth(_)
        | Command::Config(_)
        | Command::Plan { .. }
        | Command::Apply { .. }
        | Command::Completion { .. } => {}
    }
}

pub fn destructive_confirmed(command: &Command) -> bool {
    match command {
        Command::Auth(AuthCommand {
            action: AuthAction::Logout { yes },
        })
        | Command::Apply { yes, .. } => *yes,
        Command::Jira(JiraCommand {
            resource: JiraResource::Issue(IssueCommand { action }),
        }) => match action {
            IssueAction::Delete { yes, .. } => *yes,
            IssueAction::Comment {
                action: IssueCommentAction::Delete { yes, .. },
            }
            | IssueAction::Attachment {
                action: IssueAttachmentAction::Delete { yes, .. },
            }
            | IssueAction::Link {
                action: IssueLinkAction::Remove { yes, .. },
            } => *yes,
            _ => metadata(command).risk != OperationRisk::Destructive,
        },
        Command::Confluence(ConfluenceCommand {
            resource:
                ConfluenceResource::Space(SpaceCommand {
                    action: SpaceAction::Delete { yes, .. },
                })
                | ConfluenceResource::Attachment(AttachmentCommand {
                    action: AttachmentAction::Delete { yes, .. },
                })
                | ConfluenceResource::Page(PageCommand {
                    action:
                        PageAction::Delete { yes, .. }
                        | PageAction::Label {
                            action: PageLabelAction::Remove { yes, .. },
                        }
                        | PageAction::Comment {
                            action: PageCommentAction::Delete { yes, .. },
                        },
                })
                | ConfluenceResource::Blog(BlogCommand {
                    action:
                        BlogAction::Delete { yes, .. }
                        | BlogAction::Label {
                            action: BlogLabelAction::Remove { yes, .. },
                        }
                        | BlogAction::Comment {
                            action: BlogCommentAction::Delete { yes, .. },
                        },
                }),
        }) => *yes,
        _ => metadata(command).risk != OperationRisk::Destructive,
    }
}

pub fn metadata(command: &Command) -> OperationMetadata {
    match command {
        Command::Auth(command) => match &command.action {
            AuthAction::Login { .. } => write("auth.login", None),
            AuthAction::Discover { .. } => read("auth.discover", Some("GET"), false),
            AuthAction::Logout { .. } => destructive("auth.logout", "LOCAL"),
            AuthAction::Status => read("auth.status", None, false),
            AuthAction::Switch { .. } => write("auth.switch", None),
        },
        Command::Config(command) => match &command.action {
            ConfigAction::Set { .. } => write("config.set", None),
            ConfigAction::Get { .. } => read("config.get", None, false),
            ConfigAction::List => read("config.list", None, false),
        },
        Command::Jira(command) => jira_metadata(&command.resource),
        Command::Confluence(command) => confluence_metadata(&command.resource),
        Command::Plan { command, .. } => match command {
            PlannableCommand::Jira(PlanJiraCommand {
                resource:
                    PlanJiraResource::Issue(PlanIssueCommand {
                        action: PlanIssueAction::Create { .. },
                    }),
            }) => write("jira.issue.create", Some("POST")),
            PlannableCommand::Confluence(PlanConfluenceCommand {
                resource: PlanConfluenceResource::Page(PlanPageCommand { action }),
            }) => match action {
                PlanPageAction::Create { .. } => write("confluence.page.create", Some("POST")),
                PlanPageAction::Update { .. } => write("confluence.page.update", Some("PUT")),
            },
            PlannableCommand::Confluence(PlanConfluenceCommand {
                resource: PlanConfluenceResource::Blog(PlanBlogCommand { action }),
            }) => match action {
                PlanBlogAction::Create { .. } => write("confluence.blog.create", Some("POST")),
                PlanBlogAction::Update { .. } => write("confluence.blog.update", Some("PUT")),
            },
        },
        Command::Apply { .. } => destructive("plan.apply", "LOCAL"),
        Command::Completion { .. } => OperationMetadata {
            id: "completion",
            method: None,
            risk: OperationRisk::Read,
            paginated: false,
            dry_run: false,
        },
    }
}

pub fn supports_saved_plan(operation: &str) -> bool {
    matches!(
        operation,
        "jira.issue.create"
            | "confluence.page.create"
            | "confluence.page.update"
            | "confluence.blog.create"
            | "confluence.blog.update"
    )
}

fn jira_metadata(resource: &JiraResource) -> OperationMetadata {
    match resource {
        JiraResource::Issue(command) => issue_metadata(&command.action),
        JiraResource::Project(command) => match &command.action {
            ProjectAction::List { .. } => read("jira.project.list", Some("GET"), true),
            ProjectAction::View { .. } => read("jira.project.view", Some("GET"), false),
            ProjectAction::IssueTypes { .. } => {
                read("jira.project.issue-types", Some("GET"), false)
            }
        },
        JiraResource::Sprint(command) => match &command.action {
            SprintAction::List { .. } => read("jira.sprint.list", Some("GET"), true),
            SprintAction::Active { .. } => read("jira.sprint.active", Some("GET"), true),
            SprintAction::View { .. } => read("jira.sprint.view", Some("GET"), false),
            SprintAction::Create { .. } => write("jira.sprint.create", Some("POST")),
            SprintAction::Start { .. } => write("jira.sprint.start", Some("PUT")),
            SprintAction::Close { .. } => write("jira.sprint.close", Some("PUT")),
            SprintAction::Add { .. } => write("jira.sprint.add", Some("POST")),
            SprintAction::Remove { .. } => write("jira.sprint.remove", Some("POST")),
            SprintAction::Issues { .. } => read("jira.sprint.issues", Some("GET"), true),
        },
        JiraResource::Board(command) => match &command.action {
            BoardAction::List { .. } => read("jira.board.list", Some("GET"), true),
            BoardAction::View { .. } => read("jira.board.view", Some("GET"), false),
        },
        JiraResource::Search { .. } => read("jira.search", Some("GET"), true),
    }
}

fn issue_metadata(action: &IssueAction) -> OperationMetadata {
    match action {
        IssueAction::List { .. } => read("jira.issue.list", Some("GET"), true),
        IssueAction::Create { .. } => write("jira.issue.create", Some("POST")),
        IssueAction::Update { .. } => write("jira.issue.update", Some("PUT")),
        IssueAction::View { .. } => read("jira.issue.view", Some("GET"), false),
        IssueAction::Delete { .. } => destructive("jira.issue.delete", "DELETE"),
        IssueAction::Assign { .. } => write("jira.issue.assign", Some("PUT")),
        IssueAction::Transition { .. } => write("jira.issue.transition", Some("POST")),
        IssueAction::Comment { action } => match action {
            IssueCommentAction::Add { .. } => write("jira.issue.comment.add", Some("POST")),
            IssueCommentAction::List { .. } => read("jira.issue.comment.list", Some("GET"), true),
            IssueCommentAction::Update { .. } => write("jira.issue.comment.update", Some("PUT")),
            IssueCommentAction::Delete { .. } => destructive("jira.issue.comment.delete", "DELETE"),
        },
        IssueAction::Attachment { action } => match action {
            IssueAttachmentAction::Upload { .. } => {
                write("jira.issue.attachment.upload", Some("POST"))
            }
            IssueAttachmentAction::List { .. } => {
                read("jira.issue.attachment.list", Some("GET"), false)
            }
            IssueAttachmentAction::Download { .. } => {
                read("jira.issue.attachment.download", Some("GET"), false)
            }
            IssueAttachmentAction::Delete { .. } => {
                destructive("jira.issue.attachment.delete", "DELETE")
            }
        },
        IssueAction::Link { action } => match action {
            IssueLinkAction::Add { .. } => write("jira.issue.link.add", Some("POST")),
            IssueLinkAction::List { .. } => read("jira.issue.link.list", Some("GET"), false),
            IssueLinkAction::Remove { .. } => destructive("jira.issue.link.remove", "DELETE"),
            IssueLinkAction::GithubLinks { .. } => {
                read("jira.issue.link.github-links", Some("GET"), false)
            }
            IssueLinkAction::GithubCommits { .. } => {
                read("jira.issue.link.github-commits", Some("GET"), false)
            }
        },
        IssueAction::Worklog { action } => match action {
            IssueWorklogAction::Add { .. } => write("jira.issue.worklog.add", Some("POST")),
            IssueWorklogAction::List { .. } => read("jira.issue.worklog.list", Some("GET"), true),
        },
        IssueAction::Fields { .. } => read("jira.issue.fields", Some("GET"), false),
    }
}

fn confluence_metadata(resource: &ConfluenceResource) -> OperationMetadata {
    match resource {
        ConfluenceResource::Page(command) => page_metadata(&command.action),
        ConfluenceResource::Space(command) => match &command.action {
            SpaceAction::List { .. } => read("confluence.space.list", Some("GET"), true),
            SpaceAction::View { .. } => read("confluence.space.view", Some("GET"), false),
            SpaceAction::Create { .. } => write("confluence.space.create", Some("POST")),
            SpaceAction::Update { .. } => write("confluence.space.update", Some("PUT")),
            SpaceAction::Delete { .. } => destructive("confluence.space.delete", "DELETE"),
        },
        ConfluenceResource::Blog(command) => blog_metadata(&command.action),
        ConfluenceResource::Search { .. } => read("confluence.search", Some("GET"), true),
        ConfluenceResource::Attachment(command) => match &command.action {
            AttachmentAction::List { .. } => read("confluence.attachment.list", Some("GET"), true),
            AttachmentAction::View { .. } => read("confluence.attachment.view", Some("GET"), false),
            AttachmentAction::Upload { .. } => write("confluence.attachment.upload", Some("PUT")),
            AttachmentAction::Download { .. } => {
                read("confluence.attachment.download", Some("GET"), false)
            }
            AttachmentAction::Delete { .. } => {
                destructive("confluence.attachment.delete", "DELETE")
            }
        },
    }
}

fn page_metadata(action: &PageAction) -> OperationMetadata {
    match action {
        PageAction::Create { .. } => write("confluence.page.create", Some("POST")),
        PageAction::List { .. } => read("confluence.page.list", Some("GET"), true),
        PageAction::View { .. } => read("confluence.page.view", Some("GET"), false),
        PageAction::Children { .. } => read("confluence.page.children", Some("GET"), true),
        PageAction::Copy { .. } => write("confluence.page.copy", Some("POST")),
        PageAction::Update { .. } => write("confluence.page.update", Some("PUT")),
        PageAction::Delete { .. } => destructive("confluence.page.delete", "DELETE"),
        PageAction::Move { .. } => write("confluence.page.move", Some("PUT")),
        PageAction::Label { action } => match action {
            PageLabelAction::List { .. } => read("confluence.page.label.list", Some("GET"), true),
            PageLabelAction::Add { .. } => write("confluence.page.label.add", Some("POST")),
            PageLabelAction::Remove { .. } => destructive("confluence.page.label.remove", "DELETE"),
        },
        PageAction::Comment { action } => match action {
            PageCommentAction::List { .. } => {
                read("confluence.page.comment.list", Some("GET"), true)
            }
            PageCommentAction::Add { .. } => write("confluence.page.comment.add", Some("POST")),
            PageCommentAction::Delete { .. } => {
                destructive("confluence.page.comment.delete", "DELETE")
            }
        },
    }
}

fn blog_metadata(action: &BlogAction) -> OperationMetadata {
    match action {
        BlogAction::Create { .. } => write("confluence.blog.create", Some("POST")),
        BlogAction::List { .. } => read("confluence.blog.list", Some("GET"), true),
        BlogAction::View { .. } => read("confluence.blog.view", Some("GET"), false),
        BlogAction::Update { .. } => write("confluence.blog.update", Some("PUT")),
        BlogAction::Delete { .. } => destructive("confluence.blog.delete", "DELETE"),
        BlogAction::Label { action } => match action {
            BlogLabelAction::List { .. } => read("confluence.blog.label.list", Some("GET"), true),
            BlogLabelAction::Add { .. } => write("confluence.blog.label.add", Some("POST")),
            BlogLabelAction::Remove { .. } => destructive("confluence.blog.label.remove", "DELETE"),
        },
        BlogAction::Comment { action } => match action {
            BlogCommentAction::List { .. } => {
                read("confluence.blog.comment.list", Some("GET"), true)
            }
            BlogCommentAction::Add { .. } => write("confluence.blog.comment.add", Some("POST")),
            BlogCommentAction::Delete { .. } => {
                destructive("confluence.blog.comment.delete", "DELETE")
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    fn operation(args: &[&str]) -> OperationMetadata {
        let cli = Cli::try_parse_from(args).expect("command should parse");
        metadata(&cli.command)
    }

    fn confirmed(args: &[&str]) -> bool {
        let cli = Cli::try_parse_from(args).expect("command should parse");
        destructive_confirmed(&cli.command)
    }

    #[test]
    fn classifies_read_write_and_destructive_operations() {
        assert_eq!(
            operation(&["atla", "jira", "issue", "view", "PROJ-1"]).risk,
            OperationRisk::Read
        );
        assert_eq!(
            operation(&[
                "atla",
                "jira",
                "issue",
                "create",
                "--project",
                "PROJ",
                "--type",
                "Task",
                "--summary",
                "Test",
            ])
            .risk,
            OperationRisk::Write
        );
        assert_eq!(
            operation(&["atla", "confluence", "page", "delete", "123", "--yes",]).risk,
            OperationRisk::Destructive
        );
    }

    #[test]
    fn central_destructive_confirmation_covers_local_and_remote_operations() {
        assert!(!confirmed(&["atla", "auth", "logout"]));
        assert!(confirmed(&["atla", "auth", "logout", "--yes"]));
        assert!(!confirmed(&[
            "atla",
            "confluence",
            "page",
            "label",
            "remove",
            "123",
            "obsolete",
        ]));
        assert!(confirmed(&[
            "atla",
            "confluence",
            "page",
            "label",
            "remove",
            "123",
            "obsolete",
            "--yes",
        ]));
        assert!(!confirmed(&["atla", "apply", "plan.json"]));
        assert!(confirmed(&["atla", "apply", "plan.json", "--yes"]));
    }

    #[test]
    fn retry_safety_comes_from_http_method() {
        assert!(
            operation(&[
                "atla",
                "jira",
                "issue",
                "update",
                "PROJ-1",
                "--summary",
                "Updated",
            ])
            .is_retry_safe()
        );
        assert!(
            !operation(&[
                "atla",
                "jira",
                "issue",
                "create",
                "--project",
                "PROJ",
                "--type",
                "Task",
                "--summary",
                "Created",
            ])
            .is_retry_safe()
        );
    }

    #[test]
    fn list_metadata_records_pagination() {
        let metadata = operation(&["atla", "jira", "search", "project = PROJ"]);
        assert_eq!(metadata.id, "jira.search");
        assert!(metadata.paginated);
        assert_eq!(metadata.method, Some("GET"));
    }
}
