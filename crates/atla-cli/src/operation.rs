//! Central metadata registry for every CLI operation.
//!
//! Safety policy is derived from the parsed clap command rather than command
//! name heuristics, so aliases and argument values cannot bypass it.

use crate::cli::*;

mod registry;

pub use registry::{
    OPERATION_CATALOG, OperationId, OperationMetadata, OperationRisk, PlanProduct, PlanRoute,
};

pub fn catalog() -> &'static [OperationMetadata] {
    OPERATION_CATALOG
}

pub fn by_id(id: &str) -> Option<OperationMetadata> {
    catalog()
        .iter()
        .copied()
        .find(|operation| operation.id.as_str() == id)
}

fn registered(id: OperationId) -> OperationMetadata {
    by_id(id.as_str())
        .unwrap_or_else(|| panic!("operation `{id}` is missing from OPERATION_CATALOG"))
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
        | Command::Doctor(_)
        | Command::ExplainPolicy(_)
        | Command::Operation(_)
        | Command::Schema(_)
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
            AuthAction::Login { .. } => registered(OperationId::AUTH_LOGIN),
            AuthAction::Discover { .. } => registered(OperationId::AUTH_DISCOVER),
            AuthAction::Logout { .. } => registered(OperationId::AUTH_LOGOUT),
            AuthAction::Status => registered(OperationId::AUTH_STATUS),
            AuthAction::Switch { .. } => registered(OperationId::AUTH_SWITCH),
        },
        Command::Config(command) => match &command.action {
            ConfigAction::Set { .. } => registered(OperationId::CONFIG_SET),
            ConfigAction::Get { .. } => registered(OperationId::CONFIG_GET),
            ConfigAction::List => registered(OperationId::CONFIG_LIST),
        },
        Command::Jira(command) => jira_metadata(&command.resource),
        Command::Confluence(command) => confluence_metadata(&command.resource),
        Command::Doctor(_) => registered(OperationId::DOCTOR),
        Command::ExplainPolicy(_) => registered(OperationId::EXPLAIN_POLICY),
        Command::Operation(command) => match &command.action {
            OperationAction::List => registered(OperationId::OPERATION_LIST),
        },
        Command::Schema(command) => match &command.action {
            SchemaAction::List => registered(OperationId::SCHEMA_LIST),
            SchemaAction::Print { .. } => registered(OperationId::SCHEMA_PRINT),
        },
        Command::Plan { command, .. } => match command {
            PlannableCommand::Jira(PlanJiraCommand {
                resource:
                    PlanJiraResource::Issue(PlanIssueCommand {
                        action: PlanIssueAction::Create { .. },
                    }),
            }) => registered(OperationId::JIRA_ISSUE_CREATE),
            PlannableCommand::Confluence(PlanConfluenceCommand {
                resource: PlanConfluenceResource::Page(PlanPageCommand { action }),
            }) => match action {
                PlanPageAction::Create { .. } => registered(OperationId::CONFLUENCE_PAGE_CREATE),
                PlanPageAction::Update { .. } => registered(OperationId::CONFLUENCE_PAGE_UPDATE),
            },
            PlannableCommand::Confluence(PlanConfluenceCommand {
                resource: PlanConfluenceResource::Blog(PlanBlogCommand { action }),
            }) => match action {
                PlanBlogAction::Create { .. } => registered(OperationId::CONFLUENCE_BLOG_CREATE),
                PlanBlogAction::Update { .. } => registered(OperationId::CONFLUENCE_BLOG_UPDATE),
            },
        },
        Command::Apply { .. } => registered(OperationId::PLAN_APPLY),
        Command::Completion { .. } => registered(OperationId::COMPLETION),
    }
}

pub fn supports_saved_plan(operation: OperationId) -> bool {
    registered(operation).plan.is_some()
}

pub fn saved_plan_metadata(operation: &str) -> Option<OperationMetadata> {
    by_id(operation).filter(|metadata| metadata.plan.is_some())
}

fn jira_metadata(resource: &JiraResource) -> OperationMetadata {
    match resource {
        JiraResource::Issue(command) => issue_metadata(&command.action),
        JiraResource::Project(command) => match &command.action {
            ProjectAction::List { .. } => registered(OperationId::JIRA_PROJECT_LIST),
            ProjectAction::View { .. } => registered(OperationId::JIRA_PROJECT_VIEW),
            ProjectAction::IssueTypes { .. } => registered(OperationId::JIRA_PROJECT_ISSUE_TYPES),
        },
        JiraResource::Sprint(command) => match &command.action {
            SprintAction::List { .. } => registered(OperationId::JIRA_SPRINT_LIST),
            SprintAction::Active { .. } => registered(OperationId::JIRA_SPRINT_ACTIVE),
            SprintAction::View { .. } => registered(OperationId::JIRA_SPRINT_VIEW),
            SprintAction::Create { .. } => registered(OperationId::JIRA_SPRINT_CREATE),
            SprintAction::Start { .. } => registered(OperationId::JIRA_SPRINT_START),
            SprintAction::Close { .. } => registered(OperationId::JIRA_SPRINT_CLOSE),
            SprintAction::Add { .. } => registered(OperationId::JIRA_SPRINT_ADD),
            SprintAction::Remove { .. } => registered(OperationId::JIRA_SPRINT_REMOVE),
            SprintAction::Issues { .. } => registered(OperationId::JIRA_SPRINT_ISSUES),
        },
        JiraResource::Board(command) => match &command.action {
            BoardAction::List { .. } => registered(OperationId::JIRA_BOARD_LIST),
            BoardAction::View { .. } => registered(OperationId::JIRA_BOARD_VIEW),
        },
        JiraResource::Search { .. } => registered(OperationId::JIRA_SEARCH),
    }
}

fn issue_metadata(action: &IssueAction) -> OperationMetadata {
    match action {
        IssueAction::List { .. } => registered(OperationId::JIRA_ISSUE_LIST),
        IssueAction::Create { .. } => registered(OperationId::JIRA_ISSUE_CREATE),
        IssueAction::Update { .. } => registered(OperationId::JIRA_ISSUE_UPDATE),
        IssueAction::View { .. } => registered(OperationId::JIRA_ISSUE_VIEW),
        IssueAction::Delete { .. } => registered(OperationId::JIRA_ISSUE_DELETE),
        IssueAction::Assign { .. } => registered(OperationId::JIRA_ISSUE_ASSIGN),
        IssueAction::Transition { .. } => registered(OperationId::JIRA_ISSUE_TRANSITION),
        IssueAction::Comment { action } => match action {
            IssueCommentAction::Add { .. } => registered(OperationId::JIRA_ISSUE_COMMENT_ADD),
            IssueCommentAction::List { .. } => registered(OperationId::JIRA_ISSUE_COMMENT_LIST),
            IssueCommentAction::Update { .. } => registered(OperationId::JIRA_ISSUE_COMMENT_UPDATE),
            IssueCommentAction::Delete { .. } => registered(OperationId::JIRA_ISSUE_COMMENT_DELETE),
        },
        IssueAction::Attachment { action } => match action {
            IssueAttachmentAction::Upload { .. } => {
                registered(OperationId::JIRA_ISSUE_ATTACHMENT_UPLOAD)
            }
            IssueAttachmentAction::List { .. } => {
                registered(OperationId::JIRA_ISSUE_ATTACHMENT_LIST)
            }
            IssueAttachmentAction::Download { .. } => {
                registered(OperationId::JIRA_ISSUE_ATTACHMENT_DOWNLOAD)
            }
            IssueAttachmentAction::Delete { .. } => {
                registered(OperationId::JIRA_ISSUE_ATTACHMENT_DELETE)
            }
        },
        IssueAction::Link { action } => match action {
            IssueLinkAction::Add { .. } => registered(OperationId::JIRA_ISSUE_LINK_ADD),
            IssueLinkAction::List { .. } => registered(OperationId::JIRA_ISSUE_LINK_LIST),
            IssueLinkAction::Remove { .. } => registered(OperationId::JIRA_ISSUE_LINK_REMOVE),
            IssueLinkAction::GithubLinks { .. } => {
                registered(OperationId::JIRA_ISSUE_LINK_GITHUB_LINKS)
            }
            IssueLinkAction::GithubCommits { .. } => {
                registered(OperationId::JIRA_ISSUE_LINK_GITHUB_COMMITS)
            }
        },
        IssueAction::Worklog { action } => match action {
            IssueWorklogAction::Add { .. } => registered(OperationId::JIRA_ISSUE_WORKLOG_ADD),
            IssueWorklogAction::List { .. } => registered(OperationId::JIRA_ISSUE_WORKLOG_LIST),
        },
        IssueAction::Fields { .. } => registered(OperationId::JIRA_ISSUE_FIELDS),
    }
}

fn confluence_metadata(resource: &ConfluenceResource) -> OperationMetadata {
    match resource {
        ConfluenceResource::Page(command) => page_metadata(&command.action),
        ConfluenceResource::Space(command) => match &command.action {
            SpaceAction::List { .. } => registered(OperationId::CONFLUENCE_SPACE_LIST),
            SpaceAction::View { .. } => registered(OperationId::CONFLUENCE_SPACE_VIEW),
            SpaceAction::Create { .. } => registered(OperationId::CONFLUENCE_SPACE_CREATE),
            SpaceAction::Update { .. } => registered(OperationId::CONFLUENCE_SPACE_UPDATE),
            SpaceAction::Delete { .. } => registered(OperationId::CONFLUENCE_SPACE_DELETE),
        },
        ConfluenceResource::Blog(command) => blog_metadata(&command.action),
        ConfluenceResource::Search { .. } => registered(OperationId::CONFLUENCE_SEARCH),
        ConfluenceResource::Attachment(command) => match &command.action {
            AttachmentAction::List { .. } => registered(OperationId::CONFLUENCE_ATTACHMENT_LIST),
            AttachmentAction::View { .. } => registered(OperationId::CONFLUENCE_ATTACHMENT_VIEW),
            AttachmentAction::Upload { .. } => {
                registered(OperationId::CONFLUENCE_ATTACHMENT_UPLOAD)
            }
            AttachmentAction::Download { .. } => {
                registered(OperationId::CONFLUENCE_ATTACHMENT_DOWNLOAD)
            }
            AttachmentAction::Delete { .. } => {
                registered(OperationId::CONFLUENCE_ATTACHMENT_DELETE)
            }
        },
    }
}

fn page_metadata(action: &PageAction) -> OperationMetadata {
    match action {
        PageAction::Create { .. } => registered(OperationId::CONFLUENCE_PAGE_CREATE),
        PageAction::List { .. } => registered(OperationId::CONFLUENCE_PAGE_LIST),
        PageAction::View { .. } => registered(OperationId::CONFLUENCE_PAGE_VIEW),
        PageAction::Children { .. } => registered(OperationId::CONFLUENCE_PAGE_CHILDREN),
        PageAction::Copy { .. } => registered(OperationId::CONFLUENCE_PAGE_COPY),
        PageAction::Update { .. } => registered(OperationId::CONFLUENCE_PAGE_UPDATE),
        PageAction::Delete { .. } => registered(OperationId::CONFLUENCE_PAGE_DELETE),
        PageAction::Move { .. } => registered(OperationId::CONFLUENCE_PAGE_MOVE),
        PageAction::Label { action } => match action {
            PageLabelAction::List { .. } => registered(OperationId::CONFLUENCE_PAGE_LABEL_LIST),
            PageLabelAction::Add { .. } => registered(OperationId::CONFLUENCE_PAGE_LABEL_ADD),
            PageLabelAction::Remove { .. } => registered(OperationId::CONFLUENCE_PAGE_LABEL_REMOVE),
        },
        PageAction::Comment { action } => match action {
            PageCommentAction::List { .. } => registered(OperationId::CONFLUENCE_PAGE_COMMENT_LIST),
            PageCommentAction::Add { .. } => registered(OperationId::CONFLUENCE_PAGE_COMMENT_ADD),
            PageCommentAction::Delete { .. } => {
                registered(OperationId::CONFLUENCE_PAGE_COMMENT_DELETE)
            }
        },
    }
}

fn blog_metadata(action: &BlogAction) -> OperationMetadata {
    match action {
        BlogAction::Create { .. } => registered(OperationId::CONFLUENCE_BLOG_CREATE),
        BlogAction::List { .. } => registered(OperationId::CONFLUENCE_BLOG_LIST),
        BlogAction::View { .. } => registered(OperationId::CONFLUENCE_BLOG_VIEW),
        BlogAction::Update { .. } => registered(OperationId::CONFLUENCE_BLOG_UPDATE),
        BlogAction::Delete { .. } => registered(OperationId::CONFLUENCE_BLOG_DELETE),
        BlogAction::Label { action } => match action {
            BlogLabelAction::List { .. } => registered(OperationId::CONFLUENCE_BLOG_LABEL_LIST),
            BlogLabelAction::Add { .. } => registered(OperationId::CONFLUENCE_BLOG_LABEL_ADD),
            BlogLabelAction::Remove { .. } => registered(OperationId::CONFLUENCE_BLOG_LABEL_REMOVE),
        },
        BlogAction::Comment { action } => match action {
            BlogCommentAction::List { .. } => registered(OperationId::CONFLUENCE_BLOG_COMMENT_LIST),
            BlogCommentAction::Add { .. } => registered(OperationId::CONFLUENCE_BLOG_COMMENT_ADD),
            BlogCommentAction::Delete { .. } => {
                registered(OperationId::CONFLUENCE_BLOG_COMMENT_DELETE)
            }
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
            .map(|operation| (operation.id.as_str(), *operation))
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
            .map(|operation| command_path_for_operation(operation.id.as_str()))
            .collect::<BTreeSet<_>>();
        expected_paths.extend(
            catalog()
                .iter()
                .filter(|operation| supports_saved_plan(operation.id))
                .map(|operation| format!("atla plan {}", operation.id.as_str().replace('.', " "))),
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
        assert_eq!(metadata.id, OperationId::JIRA_SEARCH);
        assert!(metadata.paginated);
        assert_eq!(metadata.method, Some("GET"));
    }
}
