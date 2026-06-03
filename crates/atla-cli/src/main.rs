mod aliases;
mod cli;
mod commands;
mod config;
mod context;
mod output;
mod pagination;

use clap::{CommandFactory, Parser};
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = aliases::expand_args(std::env::args().collect())?;
    let cli = Cli::parse_from(args);

    match cli.command {
        Command::Auth(command) => commands::auth::run(command, &cli.global).await,
        Command::Config(command) => commands::config::run(command, &cli.global).await,
        Command::Jira(command) => commands::jira::run(command, &cli.global).await,
        Command::Confluence(command) => commands::confluence::run(command, &cli.global).await,
        Command::Completion { shell } => {
            let mut command = Cli::command();
            let name = command.get_name().to_owned();
            clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
            Ok(())
        }
    }
}
