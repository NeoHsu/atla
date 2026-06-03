use crate::cli::{GlobalArgs, JiraCommand, JiraResource};

mod attachment;
mod board;
mod format;
mod issue;
mod link;
mod project;
mod search;
mod sprint;
mod worklog;

pub async fn run(command: JiraCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        JiraResource::Issue(command) => issue::run_issue(command, global).await?,
        JiraResource::Project(command) => project::run_project(command, global).await?,
        JiraResource::Sprint(command) => sprint::run_sprint(command, global).await?,
        JiraResource::Board(command) => board::run_board(command, global).await?,
        JiraResource::Search {
            jql,
            limit,
            all,
            fields,
        } => search::run_search(jql, limit, all, fields, global).await?,
    }
    Ok(())
}
