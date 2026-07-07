mod aliases;
mod cli;
mod commands;
mod config;
mod context;
#[cfg(test)]
mod doc_check;
mod error;
mod output;
mod pagination;

use clap::{CommandFactory, Parser};
use cli::{Cli, Command, OutputFormat};

#[tokio::main]
async fn main() {
    let args = match aliases::expand_args(std::env::args().collect()) {
        Ok(args) => args,
        Err(err) => exit_with(err, None),
    };
    let cli = Cli::parse_from(args);
    let output = cli.global.output;

    let result = match cli.command {
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
    };

    if let Err(err) = result {
        exit_with(err, output);
    }
}

/// Report a runtime failure and exit with its classified code. With `-o json`,
/// stderr carries a machine-readable error object instead of prose.
fn exit_with(err: anyhow::Error, output: Option<OutputFormat>) -> ! {
    let classification = error::classify(&err);
    if output == Some(OutputFormat::Json) {
        let payload = serde_json::json!({
            "error": {
                "kind": classification.kind,
                "message": format!("{err:#}"),
                "status": classification.status,
                "retryable": classification.retryable,
            }
        });
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
        );
    } else {
        eprintln!("Error: {err:?}");
    }
    std::process::exit(classification.exit_code);
}
