use crate::cli::{GlobalArgs, JiraCommand, JiraResource};

pub async fn run(command: JiraCommand, _global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        JiraResource::Issue => println!("jira issue commands are planned"),
        JiraResource::Project => println!("jira project commands are planned"),
        JiraResource::Sprint => println!("jira sprint commands are planned"),
        JiraResource::Board => println!("jira board commands are planned"),
        JiraResource::Search { jql } => println!("jira search is planned: {jql}"),
    }

    Ok(())
}
