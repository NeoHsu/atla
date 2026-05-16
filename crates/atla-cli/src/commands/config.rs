use crate::cli::{ConfigAction, ConfigCommand, GlobalArgs};

pub async fn run(command: ConfigCommand, _global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        ConfigAction::Set { key, value } => println!("config set is planned: {key}={value}"),
        ConfigAction::Get { key } => println!("config get is planned: {key}"),
        ConfigAction::List => println!("config list is planned"),
    }

    Ok(())
}
