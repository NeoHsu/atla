mod cli;
mod commands;
mod config;

use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Auth(command) => commands::auth::run(command, &cli.global).await,
        Command::Config(command) => commands::config::run(command, &cli.global).await,
        Command::Jira(command) => commands::jira::run(command, &cli.global).await,
        Command::Confluence(command) => commands::confluence::run(command, &cli.global).await,
    }
}
