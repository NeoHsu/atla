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

macro_rules! operation {
    ($id:literal, $method:expr, $risk:ident, $paginated:literal, $dry_run:literal) => {
        OperationMetadata {
            id: $id,
            method: $method,
            risk: OperationRisk::$risk,
            paginated: $paginated,
            dry_run: $dry_run,
        }
    };
}

/// Complete, stable operation contract exposed to policy, tests, and discovery commands.
/// Runtime command classification resolves through this table so metadata has one source of truth.
pub const OPERATION_CATALOG: &[OperationMetadata] = &[
    operation!("auth.discover", Some("GET"), Read, false, true),
    operation!("auth.login", None, Write, false, true),
    operation!("auth.logout", Some("LOCAL"), Destructive, false, true),
    operation!("auth.status", None, Read, false, true),
    operation!("auth.switch", None, Write, false, true),
    operation!("completion", None, Read, false, false),
    operation!("config.get", None, Read, false, true),
    operation!("config.list", None, Read, false, true),
    operation!("config.set", None, Write, false, true),
    operation!(
        "confluence.attachment.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!(
        "confluence.attachment.download",
        Some("GET"),
        Read,
        false,
        true
    ),
    operation!("confluence.attachment.list", Some("GET"), Read, true, true),
    operation!(
        "confluence.attachment.upload",
        Some("PUT"),
        Write,
        false,
        true
    ),
    operation!("confluence.attachment.view", Some("GET"), Read, false, true),
    operation!(
        "confluence.blog.comment.add",
        Some("POST"),
        Write,
        false,
        true
    ),
    operation!(
        "confluence.blog.comment.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!(
        "confluence.blog.comment.list",
        Some("GET"),
        Read,
        true,
        true
    ),
    operation!("confluence.blog.create", Some("POST"), Write, false, true),
    operation!(
        "confluence.blog.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!(
        "confluence.blog.label.add",
        Some("POST"),
        Write,
        false,
        true
    ),
    operation!("confluence.blog.label.list", Some("GET"), Read, true, true),
    operation!(
        "confluence.blog.label.remove",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!("confluence.blog.list", Some("GET"), Read, true, true),
    operation!("confluence.blog.update", Some("PUT"), Write, false, true),
    operation!("confluence.blog.view", Some("GET"), Read, false, true),
    operation!("confluence.page.children", Some("GET"), Read, true, true),
    operation!(
        "confluence.page.comment.add",
        Some("POST"),
        Write,
        false,
        true
    ),
    operation!(
        "confluence.page.comment.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!(
        "confluence.page.comment.list",
        Some("GET"),
        Read,
        true,
        true
    ),
    operation!("confluence.page.copy", Some("POST"), Write, false, true),
    operation!("confluence.page.create", Some("POST"), Write, false, true),
    operation!(
        "confluence.page.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!(
        "confluence.page.label.add",
        Some("POST"),
        Write,
        false,
        true
    ),
    operation!("confluence.page.label.list", Some("GET"), Read, true, true),
    operation!(
        "confluence.page.label.remove",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!("confluence.page.list", Some("GET"), Read, true, true),
    operation!("confluence.page.move", Some("PUT"), Write, false, true),
    operation!("confluence.page.update", Some("PUT"), Write, false, true),
    operation!("confluence.page.view", Some("GET"), Read, false, true),
    operation!("confluence.search", Some("GET"), Read, true, true),
    operation!("confluence.space.create", Some("POST"), Write, false, true),
    operation!(
        "confluence.space.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!("confluence.space.list", Some("GET"), Read, true, true),
    operation!("confluence.space.update", Some("PUT"), Write, false, true),
    operation!("confluence.space.view", Some("GET"), Read, false, true),
    operation!("jira.board.list", Some("GET"), Read, true, true),
    operation!("jira.board.view", Some("GET"), Read, false, true),
    operation!("jira.issue.assign", Some("PUT"), Write, false, true),
    operation!(
        "jira.issue.attachment.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!(
        "jira.issue.attachment.download",
        Some("GET"),
        Read,
        false,
        true
    ),
    operation!("jira.issue.attachment.list", Some("GET"), Read, false, true),
    operation!(
        "jira.issue.attachment.upload",
        Some("POST"),
        Write,
        false,
        true
    ),
    operation!("jira.issue.comment.add", Some("POST"), Write, false, true),
    operation!(
        "jira.issue.comment.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!("jira.issue.comment.list", Some("GET"), Read, true, true),
    operation!("jira.issue.comment.update", Some("PUT"), Write, false, true),
    operation!("jira.issue.create", Some("POST"), Write, false, true),
    operation!(
        "jira.issue.delete",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!("jira.issue.fields", Some("GET"), Read, false, true),
    operation!("jira.issue.link.add", Some("POST"), Write, false, true),
    operation!(
        "jira.issue.link.github-commits",
        Some("GET"),
        Read,
        false,
        true
    ),
    operation!(
        "jira.issue.link.github-links",
        Some("GET"),
        Read,
        false,
        true
    ),
    operation!("jira.issue.link.list", Some("GET"), Read, false, true),
    operation!(
        "jira.issue.link.remove",
        Some("DELETE"),
        Destructive,
        false,
        true
    ),
    operation!("jira.issue.list", Some("GET"), Read, true, true),
    operation!("jira.issue.transition", Some("POST"), Write, false, true),
    operation!("jira.issue.update", Some("PUT"), Write, false, true),
    operation!("jira.issue.view", Some("GET"), Read, false, true),
    operation!("jira.issue.worklog.add", Some("POST"), Write, false, true),
    operation!("jira.issue.worklog.list", Some("GET"), Read, true, true),
    operation!("jira.project.issue-types", Some("GET"), Read, false, true),
    operation!("jira.project.list", Some("GET"), Read, true, true),
    operation!("jira.project.view", Some("GET"), Read, false, true),
    operation!("jira.search", Some("GET"), Read, true, true),
    operation!("jira.sprint.active", Some("GET"), Read, true, true),
    operation!("jira.sprint.add", Some("POST"), Write, false, true),
    operation!("jira.sprint.close", Some("PUT"), Write, false, true),
    operation!("jira.sprint.create", Some("POST"), Write, false, true),
    operation!("jira.sprint.issues", Some("GET"), Read, true, true),
    operation!("jira.sprint.list", Some("GET"), Read, true, true),
    operation!("jira.sprint.remove", Some("POST"), Write, false, true),
    operation!("jira.sprint.start", Some("PUT"), Write, false, true),
    operation!("jira.sprint.view", Some("GET"), Read, false, true),
    operation!("plan.apply", Some("LOCAL"), Destructive, false, true),
];

pub fn catalog() -> &'static [OperationMetadata] {
    OPERATION_CATALOG
}

pub fn by_id(id: &str) -> Option<OperationMetadata> {
    catalog()
        .iter()
        .copied()
        .find(|operation| operation.id == id)
}

fn registered(id: &'static str) -> OperationMetadata {
    by_id(id).unwrap_or_else(|| panic!("operation `{id}` is missing from OPERATION_CATALOG"))
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
            AuthAction::Login { .. } => registered("auth.login"),
            AuthAction::Discover { .. } => registered("auth.discover"),
            AuthAction::Logout { .. } => registered("auth.logout"),
            AuthAction::Status => registered("auth.status"),
            AuthAction::Switch { .. } => registered("auth.switch"),
        },
        Command::Config(command) => match &command.action {
            ConfigAction::Set { .. } => registered("config.set"),
            ConfigAction::Get { .. } => registered("config.get"),
            ConfigAction::List => registered("config.list"),
        },
        Command::Jira(command) => jira_metadata(&command.resource),
        Command::Confluence(command) => confluence_metadata(&command.resource),
        Command::Plan { command, .. } => match command {
            PlannableCommand::Jira(PlanJiraCommand {
                resource:
                    PlanJiraResource::Issue(PlanIssueCommand {
                        action: PlanIssueAction::Create { .. },
                    }),
            }) => registered("jira.issue.create"),
            PlannableCommand::Confluence(PlanConfluenceCommand {
                resource: PlanConfluenceResource::Page(PlanPageCommand { action }),
            }) => match action {
                PlanPageAction::Create { .. } => registered("confluence.page.create"),
                PlanPageAction::Update { .. } => registered("confluence.page.update"),
            },
            PlannableCommand::Confluence(PlanConfluenceCommand {
                resource: PlanConfluenceResource::Blog(PlanBlogCommand { action }),
            }) => match action {
                PlanBlogAction::Create { .. } => registered("confluence.blog.create"),
                PlanBlogAction::Update { .. } => registered("confluence.blog.update"),
            },
        },
        Command::Apply { .. } => registered("plan.apply"),
        Command::Completion { .. } => registered("completion"),
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
            ProjectAction::List { .. } => registered("jira.project.list"),
            ProjectAction::View { .. } => registered("jira.project.view"),
            ProjectAction::IssueTypes { .. } => registered("jira.project.issue-types"),
        },
        JiraResource::Sprint(command) => match &command.action {
            SprintAction::List { .. } => registered("jira.sprint.list"),
            SprintAction::Active { .. } => registered("jira.sprint.active"),
            SprintAction::View { .. } => registered("jira.sprint.view"),
            SprintAction::Create { .. } => registered("jira.sprint.create"),
            SprintAction::Start { .. } => registered("jira.sprint.start"),
            SprintAction::Close { .. } => registered("jira.sprint.close"),
            SprintAction::Add { .. } => registered("jira.sprint.add"),
            SprintAction::Remove { .. } => registered("jira.sprint.remove"),
            SprintAction::Issues { .. } => registered("jira.sprint.issues"),
        },
        JiraResource::Board(command) => match &command.action {
            BoardAction::List { .. } => registered("jira.board.list"),
            BoardAction::View { .. } => registered("jira.board.view"),
        },
        JiraResource::Search { .. } => registered("jira.search"),
    }
}

fn issue_metadata(action: &IssueAction) -> OperationMetadata {
    match action {
        IssueAction::List { .. } => registered("jira.issue.list"),
        IssueAction::Create { .. } => registered("jira.issue.create"),
        IssueAction::Update { .. } => registered("jira.issue.update"),
        IssueAction::View { .. } => registered("jira.issue.view"),
        IssueAction::Delete { .. } => registered("jira.issue.delete"),
        IssueAction::Assign { .. } => registered("jira.issue.assign"),
        IssueAction::Transition { .. } => registered("jira.issue.transition"),
        IssueAction::Comment { action } => match action {
            IssueCommentAction::Add { .. } => registered("jira.issue.comment.add"),
            IssueCommentAction::List { .. } => registered("jira.issue.comment.list"),
            IssueCommentAction::Update { .. } => registered("jira.issue.comment.update"),
            IssueCommentAction::Delete { .. } => registered("jira.issue.comment.delete"),
        },
        IssueAction::Attachment { action } => match action {
            IssueAttachmentAction::Upload { .. } => registered("jira.issue.attachment.upload"),
            IssueAttachmentAction::List { .. } => registered("jira.issue.attachment.list"),
            IssueAttachmentAction::Download { .. } => registered("jira.issue.attachment.download"),
            IssueAttachmentAction::Delete { .. } => registered("jira.issue.attachment.delete"),
        },
        IssueAction::Link { action } => match action {
            IssueLinkAction::Add { .. } => registered("jira.issue.link.add"),
            IssueLinkAction::List { .. } => registered("jira.issue.link.list"),
            IssueLinkAction::Remove { .. } => registered("jira.issue.link.remove"),
            IssueLinkAction::GithubLinks { .. } => registered("jira.issue.link.github-links"),
            IssueLinkAction::GithubCommits { .. } => registered("jira.issue.link.github-commits"),
        },
        IssueAction::Worklog { action } => match action {
            IssueWorklogAction::Add { .. } => registered("jira.issue.worklog.add"),
            IssueWorklogAction::List { .. } => registered("jira.issue.worklog.list"),
        },
        IssueAction::Fields { .. } => registered("jira.issue.fields"),
    }
}

fn confluence_metadata(resource: &ConfluenceResource) -> OperationMetadata {
    match resource {
        ConfluenceResource::Page(command) => page_metadata(&command.action),
        ConfluenceResource::Space(command) => match &command.action {
            SpaceAction::List { .. } => registered("confluence.space.list"),
            SpaceAction::View { .. } => registered("confluence.space.view"),
            SpaceAction::Create { .. } => registered("confluence.space.create"),
            SpaceAction::Update { .. } => registered("confluence.space.update"),
            SpaceAction::Delete { .. } => registered("confluence.space.delete"),
        },
        ConfluenceResource::Blog(command) => blog_metadata(&command.action),
        ConfluenceResource::Search { .. } => registered("confluence.search"),
        ConfluenceResource::Attachment(command) => match &command.action {
            AttachmentAction::List { .. } => registered("confluence.attachment.list"),
            AttachmentAction::View { .. } => registered("confluence.attachment.view"),
            AttachmentAction::Upload { .. } => registered("confluence.attachment.upload"),
            AttachmentAction::Download { .. } => registered("confluence.attachment.download"),
            AttachmentAction::Delete { .. } => registered("confluence.attachment.delete"),
        },
    }
}

fn page_metadata(action: &PageAction) -> OperationMetadata {
    match action {
        PageAction::Create { .. } => registered("confluence.page.create"),
        PageAction::List { .. } => registered("confluence.page.list"),
        PageAction::View { .. } => registered("confluence.page.view"),
        PageAction::Children { .. } => registered("confluence.page.children"),
        PageAction::Copy { .. } => registered("confluence.page.copy"),
        PageAction::Update { .. } => registered("confluence.page.update"),
        PageAction::Delete { .. } => registered("confluence.page.delete"),
        PageAction::Move { .. } => registered("confluence.page.move"),
        PageAction::Label { action } => match action {
            PageLabelAction::List { .. } => registered("confluence.page.label.list"),
            PageLabelAction::Add { .. } => registered("confluence.page.label.add"),
            PageLabelAction::Remove { .. } => registered("confluence.page.label.remove"),
        },
        PageAction::Comment { action } => match action {
            PageCommentAction::List { .. } => registered("confluence.page.comment.list"),
            PageCommentAction::Add { .. } => registered("confluence.page.comment.add"),
            PageCommentAction::Delete { .. } => registered("confluence.page.comment.delete"),
        },
    }
}

fn blog_metadata(action: &BlogAction) -> OperationMetadata {
    match action {
        BlogAction::Create { .. } => registered("confluence.blog.create"),
        BlogAction::List { .. } => registered("confluence.blog.list"),
        BlogAction::View { .. } => registered("confluence.blog.view"),
        BlogAction::Update { .. } => registered("confluence.blog.update"),
        BlogAction::Delete { .. } => registered("confluence.blog.delete"),
        BlogAction::Label { action } => match action {
            BlogLabelAction::List { .. } => registered("confluence.blog.label.list"),
            BlogLabelAction::Add { .. } => registered("confluence.blog.label.add"),
            BlogLabelAction::Remove { .. } => registered("confluence.blog.label.remove"),
        },
        BlogAction::Comment { action } => match action {
            BlogCommentAction::List { .. } => registered("confluence.blog.comment.list"),
            BlogCommentAction::Add { .. } => registered("confluence.blog.comment.add"),
            BlogCommentAction::Delete { .. } => registered("confluence.blog.comment.delete"),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use clap::{CommandFactory, Parser};

    use super::*;

    #[derive(Debug)]
    struct LeafCommand {
        path: String,
        arguments: BTreeSet<String>,
    }

    fn collect_leaf_commands(command: &clap::Command, path: &str, leaves: &mut Vec<LeafCommand>) {
        let subcommands = command
            .get_subcommands()
            .filter(|subcommand| subcommand.get_name() != "help")
            .collect::<Vec<_>>();
        if subcommands.is_empty() {
            leaves.push(LeafCommand {
                path: path.to_owned(),
                arguments: command
                    .get_arguments()
                    .filter(|argument| !argument.is_global_set())
                    .map(|argument| argument.get_id().to_string())
                    .collect(),
            });
            return;
        }
        for subcommand in subcommands {
            collect_leaf_commands(
                subcommand,
                &format!("{path} {}", subcommand.get_name()),
                leaves,
            );
        }
    }

    fn operation_id_for_path(path: &str) -> String {
        let path = path.strip_prefix("atla ").expect("atla command path");
        match path {
            "apply" => "plan.apply".to_owned(),
            "completion" => "completion".to_owned(),
            path if path.starts_with("plan ") => path[5..].replace(' ', "."),
            path => path.replace(' ', "."),
        }
    }

    fn command_path_for_operation(operation: &str) -> String {
        match operation {
            "plan.apply" => "atla apply".to_owned(),
            "completion" => "atla completion".to_owned(),
            operation => format!("atla {}", operation.replace('.', " ")),
        }
    }

    fn operation(args: &[&str]) -> OperationMetadata {
        let cli = Cli::try_parse_from(args).expect("command should parse");
        metadata(&cli.command)
    }

    fn confirmed(args: &[&str]) -> bool {
        let cli = Cli::try_parse_from(args).expect("command should parse");
        destructive_confirmed(&cli.command)
    }

    #[test]
    fn catalog_covers_every_cli_leaf_and_safety_marker() {
        let catalog_by_id = catalog()
            .iter()
            .map(|operation| (operation.id, *operation))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            catalog_by_id.len(),
            catalog().len(),
            "operation IDs must be unique"
        );

        let mut command = Cli::command();
        command.build();
        let mut leaves = Vec::new();
        collect_leaf_commands(&command, command.get_name(), &mut leaves);

        let actual_paths = leaves
            .iter()
            .map(|leaf| leaf.path.clone())
            .collect::<BTreeSet<_>>();
        let mut expected_paths = catalog()
            .iter()
            .map(|operation| command_path_for_operation(operation.id))
            .collect::<BTreeSet<_>>();
        expected_paths.extend(
            catalog()
                .iter()
                .filter(|operation| supports_saved_plan(operation.id))
                .map(|operation| format!("atla plan {}", operation.id.replace('.', " "))),
        );
        assert_eq!(
            actual_paths, expected_paths,
            "every clap leaf must have one registered operation contract"
        );

        for leaf in leaves {
            let operation_id = operation_id_for_path(&leaf.path);
            let operation = catalog_by_id
                .get(operation_id.as_str())
                .unwrap_or_else(|| panic!("{} has no operation metadata", leaf.path));
            let has_yes = leaf.arguments.contains("yes");
            assert_eq!(
                has_yes,
                operation.risk == OperationRisk::Destructive,
                "{} destructive confirmation marker drifted",
                leaf.path
            );
            let has_pagination = leaf.arguments.contains("page_token");
            assert_eq!(
                has_pagination, operation.paginated,
                "{} pagination marker drifted",
                leaf.path
            );
            if operation.paginated {
                assert!(
                    leaf.arguments.contains("limit"),
                    "{} is paginated but has no limit",
                    leaf.path
                );
            }
            if operation_id.starts_with("jira.")
                || operation_id.starts_with("confluence.")
                || operation_id == "auth.discover"
            {
                assert!(
                    operation.method.is_some_and(|method| method != "LOCAL"),
                    "{} network operation has no HTTP method",
                    leaf.path
                );
            }
        }
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
