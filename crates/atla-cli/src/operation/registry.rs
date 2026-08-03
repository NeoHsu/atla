//! Typed, declarative registry for every stable CLI operation.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(&'static str);

impl OperationId {
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl AsRef<str> for OperationId {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

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
pub enum PlanProduct {
    Jira,
    Confluence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanRoute {
    Exact(&'static str),
    NumericResource {
        prefix: &'static str,
        allow_title_suffix: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanContract {
    pub product: PlanProduct,
    pub route: PlanRoute,
    pub allowed_true_queries: &'static [&'static str],
}

impl PlanContract {
    const fn jira(route: PlanRoute, allowed_true_queries: &'static [&'static str]) -> Self {
        Self {
            product: PlanProduct::Jira,
            route,
            allowed_true_queries,
        }
    }

    const fn confluence(route: PlanRoute, allowed_true_queries: &'static [&'static str]) -> Self {
        Self {
            product: PlanProduct::Confluence,
            route,
            allowed_true_queries,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationMetadata {
    pub id: OperationId,
    pub method: Option<&'static str>,
    pub risk: OperationRisk,
    pub paginated: bool,
    pub dry_run: bool,
    pub plan: Option<PlanContract>,
}

impl OperationMetadata {
    pub fn is_retry_safe(self) -> bool {
        matches!(
            self.method,
            Some("GET" | "HEAD" | "PUT" | "DELETE" | "OPTIONS" | "TRACE")
        )
    }
}

macro_rules! define_operations {
    ($(
        $name:ident => {
            id: $id:literal,
            method: $method:expr,
            risk: $risk:ident,
            paginated: $paginated:literal,
            dry_run: $dry_run:literal,
            plan: $plan:expr
        };
    )+) => {
        impl OperationId {
            $(pub const $name: Self = Self($id);)+
        }

        /// Complete, stable operation contract exposed to policy, tests, and discovery commands.
        pub const OPERATION_CATALOG: &[OperationMetadata] = &[
            $(OperationMetadata {
                id: OperationId::$name,
                method: $method,
                risk: OperationRisk::$risk,
                paginated: $paginated,
                dry_run: $dry_run,
                plan: $plan,
            },)+
        ];
    };
}

define_operations! {
    AUTH_DISCOVER => { id: "auth.discover", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    AUTH_LOGIN => { id: "auth.login", method: None, risk: Write, paginated: false, dry_run: true, plan: None };
    AUTH_LOGOUT => { id: "auth.logout", method: Some("LOCAL"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    AUTH_STATUS => { id: "auth.status", method: None, risk: Read, paginated: false, dry_run: true, plan: None };
    AUTH_SWITCH => { id: "auth.switch", method: None, risk: Write, paginated: false, dry_run: true, plan: None };
    COMPLETION => { id: "completion", method: None, risk: Read, paginated: false, dry_run: false, plan: None };
    CONFIG_GET => { id: "config.get", method: None, risk: Read, paginated: false, dry_run: true, plan: None };
    CONFIG_LIST => { id: "config.list", method: None, risk: Read, paginated: false, dry_run: true, plan: None };
    CONFIG_SET => { id: "config.set", method: None, risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_ATTACHMENT_DELETE => { id: "confluence.attachment.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_ATTACHMENT_DOWNLOAD => { id: "confluence.attachment.download", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_ATTACHMENT_LIST => { id: "confluence.attachment.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_ATTACHMENT_UPLOAD => { id: "confluence.attachment.upload", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_ATTACHMENT_VIEW => { id: "confluence.attachment.view", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_BLOG_COMMENT_ADD => { id: "confluence.blog.comment.add", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_BLOG_COMMENT_DELETE => { id: "confluence.blog.comment.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_BLOG_COMMENT_LIST => { id: "confluence.blog.comment.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_BLOG_CREATE => { id: "confluence.blog.create", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: Some(PlanContract::confluence(PlanRoute::Exact("/wiki/api/v2/blogposts"), &["private"])) };
    CONFLUENCE_BLOG_DELETE => { id: "confluence.blog.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_BLOG_LABEL_ADD => { id: "confluence.blog.label.add", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_BLOG_LABEL_LIST => { id: "confluence.blog.label.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_BLOG_LABEL_REMOVE => { id: "confluence.blog.label.remove", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_BLOG_LIST => { id: "confluence.blog.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_BLOG_UPDATE => { id: "confluence.blog.update", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: Some(PlanContract::confluence(PlanRoute::NumericResource { prefix: "/wiki/api/v2/blogposts/", allow_title_suffix: false }, &[])) };
    CONFLUENCE_BLOG_VIEW => { id: "confluence.blog.view", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_PAGE_CHILDREN => { id: "confluence.page.children", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_PAGE_COMMENT_ADD => { id: "confluence.page.comment.add", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_PAGE_COMMENT_DELETE => { id: "confluence.page.comment.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_PAGE_COMMENT_LIST => { id: "confluence.page.comment.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_PAGE_COPY => { id: "confluence.page.copy", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_PAGE_CREATE => { id: "confluence.page.create", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: Some(PlanContract::confluence(PlanRoute::Exact("/wiki/api/v2/pages"), &["private", "root-level"])) };
    CONFLUENCE_PAGE_DELETE => { id: "confluence.page.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_PAGE_LABEL_ADD => { id: "confluence.page.label.add", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_PAGE_LABEL_LIST => { id: "confluence.page.label.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_PAGE_LABEL_REMOVE => { id: "confluence.page.label.remove", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_PAGE_LIST => { id: "confluence.page.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_PAGE_MOVE => { id: "confluence.page.move", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_PAGE_UPDATE => { id: "confluence.page.update", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: Some(PlanContract::confluence(PlanRoute::NumericResource { prefix: "/wiki/api/v2/pages/", allow_title_suffix: true }, &[])) };
    CONFLUENCE_PAGE_VIEW => { id: "confluence.page.view", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_SEARCH => { id: "confluence.search", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_SPACE_CREATE => { id: "confluence.space.create", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_SPACE_DELETE => { id: "confluence.space.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_SPACE_LIST => { id: "confluence.space.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    CONFLUENCE_SPACE_UPDATE => { id: "confluence.space.update", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: None };
    CONFLUENCE_SPACE_VIEW => { id: "confluence.space.view", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    DOCTOR => { id: "doctor", method: Some("GET"), risk: Read, paginated: false, dry_run: false, plan: None };
    EXPLAIN_POLICY => { id: "explain-policy", method: Some("LOCAL"), risk: Read, paginated: false, dry_run: false, plan: None };
    JIRA_BOARD_LIST => { id: "jira.board.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_BOARD_VIEW => { id: "jira.board.view", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_ASSIGN => { id: "jira.issue.assign", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_ATTACHMENT_DELETE => { id: "jira.issue.attachment.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_ATTACHMENT_DOWNLOAD => { id: "jira.issue.attachment.download", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_ATTACHMENT_LIST => { id: "jira.issue.attachment.list", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_ATTACHMENT_UPLOAD => { id: "jira.issue.attachment.upload", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_COMMENT_ADD => { id: "jira.issue.comment.add", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_COMMENT_DELETE => { id: "jira.issue.comment.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_COMMENT_LIST => { id: "jira.issue.comment.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_ISSUE_COMMENT_UPDATE => { id: "jira.issue.comment.update", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_CREATE => { id: "jira.issue.create", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: Some(PlanContract::jira(PlanRoute::Exact("/rest/api/3/issue"), &[])) };
    JIRA_ISSUE_DELETE => { id: "jira.issue.delete", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_FIELDS => { id: "jira.issue.fields", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_LINK_ADD => { id: "jira.issue.link.add", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_LINK_GITHUB_COMMITS => { id: "jira.issue.link.github-commits", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_LINK_GITHUB_LINKS => { id: "jira.issue.link.github-links", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_LINK_LIST => { id: "jira.issue.link.list", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_LINK_REMOVE => { id: "jira.issue.link.remove", method: Some("DELETE"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_LIST => { id: "jira.issue.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_ISSUE_TRANSITION => { id: "jira.issue.transition", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_UPDATE => { id: "jira.issue.update", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_VIEW => { id: "jira.issue.view", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_WORKLOG_ADD => { id: "jira.issue.worklog.add", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_ISSUE_WORKLOG_LIST => { id: "jira.issue.worklog.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_PROJECT_ISSUE_TYPES => { id: "jira.project.issue-types", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_PROJECT_LIST => { id: "jira.project.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_PROJECT_VIEW => { id: "jira.project.view", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    JIRA_SEARCH => { id: "jira.search", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_SPRINT_ACTIVE => { id: "jira.sprint.active", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_SPRINT_ADD => { id: "jira.sprint.add", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_SPRINT_CLOSE => { id: "jira.sprint.close", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_SPRINT_CREATE => { id: "jira.sprint.create", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_SPRINT_ISSUES => { id: "jira.sprint.issues", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_SPRINT_LIST => { id: "jira.sprint.list", method: Some("GET"), risk: Read, paginated: true, dry_run: true, plan: None };
    JIRA_SPRINT_REMOVE => { id: "jira.sprint.remove", method: Some("POST"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_SPRINT_START => { id: "jira.sprint.start", method: Some("PUT"), risk: Write, paginated: false, dry_run: true, plan: None };
    JIRA_SPRINT_VIEW => { id: "jira.sprint.view", method: Some("GET"), risk: Read, paginated: false, dry_run: true, plan: None };
    OPERATION_LIST => { id: "operation.list", method: Some("LOCAL"), risk: Read, paginated: false, dry_run: false, plan: None };
    PLAN_APPLY => { id: "plan.apply", method: Some("LOCAL"), risk: Destructive, paginated: false, dry_run: true, plan: None };
    SCHEMA_LIST => { id: "schema.list", method: Some("LOCAL"), risk: Read, paginated: false, dry_run: false, plan: None };
    SCHEMA_PRINT => { id: "schema.print", method: Some("LOCAL"), risk: Read, paginated: false, dry_run: false, plan: None };
}
