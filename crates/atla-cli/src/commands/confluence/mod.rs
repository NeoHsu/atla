use crate::cli::{ConfluenceCommand, ConfluenceResource, GlobalArgs};

mod attachment;
mod blog;
mod blog_comment;
mod blog_label;
mod format;
mod page;
mod page_comment;
mod page_label;
mod search;
mod space;

pub async fn run(command: ConfluenceCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        ConfluenceResource::Page(command) => page::run_page(command, global).await?,
        ConfluenceResource::Space(command) => space::run_space(command, global).await?,
        ConfluenceResource::Blog(command) => blog::run_blog(command, global).await?,
        ConfluenceResource::Search { cql, limit, all } => {
            search::run_search(cql, limit, all, global).await?
        }
        ConfluenceResource::Attachment(command) => {
            attachment::run_attachment(command, global).await?
        }
    }

    Ok(())
}
