use crate::cli::{ConfluenceCommand, ConfluenceResource, GlobalArgs};

pub async fn run(command: ConfluenceCommand, _global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        ConfluenceResource::Page => println!("confluence page commands are planned"),
        ConfluenceResource::Space => println!("confluence space commands are planned"),
        ConfluenceResource::Blog => println!("confluence blog commands are planned"),
        ConfluenceResource::Search { cql } => println!("confluence search is planned: {cql}"),
        ConfluenceResource::Attachment => println!("confluence attachment commands are planned"),
    }

    Ok(())
}
