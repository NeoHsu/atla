mod aliases;
mod cli;
mod commands;
mod config;
mod context;
#[cfg(test)]
mod doc_check;
mod error;
mod operation;
mod output;
mod pagination;
mod policy;

use clap::{CommandFactory, Parser};
use cli::{Cli, Command, OutputFormat};

#[tokio::main]
async fn main() {
    let args = match aliases::expand_args(std::env::args().collect()) {
        Ok(args) => args,
        Err(err) => exit_with(err, None, None),
    };
    let mut cli = Cli::parse_from(args);
    if matches!(&cli.command, Command::Plan { .. } | Command::Apply { .. }) {
        if cli
            .global
            .output
            .is_some_and(|format| format != OutputFormat::Json)
        {
            exit_with(
                anyhow::Error::new(error::UsageError(
                    "plan/apply support only --output json".to_owned(),
                )),
                cli.global.output,
                None,
            );
        }
        cli.global.output = Some(OutputFormat::Json);
    }
    if cli.global.max_bytes.is_some() && cli.global.output != Some(OutputFormat::Json) {
        exit_with(
            anyhow::Error::new(error::UsageError(
                "--max-bytes requires --output json".to_owned(),
            )),
            cli.global.output,
            None,
        );
    }
    if let Command::Plan {
        out, expires_in, ..
    } = &cli.command
    {
        cli.global.dry_run = true;
        let out = out.clone().unwrap_or_else(|| {
            exit_with(
                anyhow::Error::new(error::UsageError("plan requires --out <PATH>".to_owned())),
                Some(OutputFormat::Json),
                None,
            )
        });
        output::configure_plan_output(out, *expires_in);
    }
    let output = cli.global.output;
    output::configure_max_bytes(cli.global.max_bytes);
    operation::apply_context_budgets(
        &mut cli.command,
        cli.global.max_pages.is_some() || cli.global.max_items.is_some(),
    );
    let operation = operation::metadata(&cli.command);
    output::configure_operation(operation.id, operation.risk.mutates(), cli.global.dry_run);
    if matches!(&cli.command, Command::Plan { .. }) && !operation::supports_saved_plan(operation.id)
    {
        exit_with(
            anyhow::Error::new(error::UsageError(format!(
                "operation `{}` does not yet support saved plans",
                operation.id
            ))),
            output,
            Some(operation),
        );
    }
    if cli.global.dry_run
        && output == Some(OutputFormat::Json)
        && !operation::supports_saved_plan(operation.id)
    {
        exit_with(
            anyhow::Error::new(error::UsageError(format!(
                "operation `{}` does not support structured JSON dry-run; omit --output json",
                operation.id
            ))),
            output,
            Some(operation),
        );
    }
    if cli.global.verbose {
        eprintln!(
            "[verbose] operation={} method={} risk={:?} paginated={} dry-run-supported={}",
            operation.id,
            operation.method.unwrap_or("LOCAL"),
            operation.risk,
            operation.paginated,
            operation.dry_run
        );
    }
    if cli.global.read_only
        && operation.risk.mutates()
        && (!cli.global.dry_run || matches!(&cli.command, Command::Plan { .. }))
    {
        exit_with(
            anyhow::Error::new(error::UsageError(format!(
                "operation `{}` is blocked by --read-only",
                operation.id
            ))),
            output,
            Some(operation),
        );
    }
    if operation.risk == operation::OperationRisk::Destructive
        && !cli.global.dry_run
        && !operation::destructive_confirmed(&cli.command)
    {
        exit_with(
            anyhow::Error::new(error::UsageError(format!(
                "operation `{}` requires --yes",
                operation.id
            ))),
            output,
            Some(operation),
        );
    }
    if let Err(error) = policy::enforce_profile_policy(&cli.global, operation) {
        exit_with(error, output, Some(operation));
    }

    if let Err(err) = run_command(cli.command, &cli.global).await {
        exit_with(err, output, Some(operation));
    }
}

/// Dispatch one parsed command while keeping handler futures off the main
/// thread's stack. Storing every handler future inline here overflows the
/// smaller default stack used by Windows binaries.
async fn run_command(command: Command, global: &cli::GlobalArgs) -> anyhow::Result<()> {
    match command {
        Command::Auth(command) => Box::pin(commands::auth::run(command, global)).await,
        Command::Config(command) => Box::pin(commands::config::run(command, global)).await,
        Command::Jira(command) => Box::pin(commands::jira::run(command, global)).await,
        Command::Confluence(command) => Box::pin(commands::confluence::run(command, global)).await,
        Command::Doctor(args) => Box::pin(commands::discovery::doctor(args, global)).await,
        Command::ExplainPolicy(args) => commands::discovery::explain_policy(args, global),
        Command::Operation(command) => commands::discovery::operation(command, global),
        Command::Schema(command) => commands::discovery::schema(command, global),
        Command::Plan { command, .. } => match command.into_command() {
            Command::Jira(command) => Box::pin(commands::jira::run(command, global)).await,
            Command::Confluence(command) => {
                Box::pin(commands::confluence::run(command, global)).await
            }
            _ => unreachable!("plannable commands convert only to product commands"),
        },
        Command::Apply { plan, yes } => Box::pin(commands::plan::apply(&plan, yes, global)).await,
        Command::Completion { shell } => {
            let mut command = Cli::command();
            let name = command.get_name().to_owned();
            clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
            Ok(())
        }
    }
}

/// Report a runtime failure and exit with its classified code. With `-o json`,
/// stderr carries a machine-readable error object instead of prose.
fn exit_with(
    err: anyhow::Error,
    output: Option<OutputFormat>,
    operation: Option<operation::OperationMetadata>,
) -> ! {
    let mut classification = error::classify(&err);
    if operation.is_some_and(|operation| operation.risk.mutates() && !operation.is_retry_safe())
        && classification.retryable
        && classification.status != Some(429)
    {
        classification.exit_code = 1;
        classification.kind = "ambiguous_mutation";
        classification.retryable = false;
    }
    if output == Some(OutputFormat::Json) {
        let payload = output::schema::ErrorEnvelope {
            schema_version: output::schema::SCHEMA_VERSION,
            error: output::schema::ErrorBody {
                kind: classification.kind,
                message: format!("{err:#}"),
                status: classification.status,
                retryable: classification.retryable,
            },
        };
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| {
                "{\"schemaVersion\":1,\"error\":{\"kind\":\"serialization\",\"message\":\"failed to encode error\",\"status\":null,\"retryable\":false}}".to_owned()
            })
        );
    } else {
        eprintln!("Error: {err:?}");
    }
    std::process::exit(classification.exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_dispatch_future_stays_small() {
        let Cli { global, command } = Cli::try_parse_from(["atla", "schema", "list"])
            .expect("representative command should parse");
        let future = run_command(command, &global);

        assert!(
            std::mem::size_of_val(&future) <= 4 * 1024,
            "command dispatch future grew to {} bytes; box handler futures to protect the Windows main stack",
            std::mem::size_of_val(&future)
        );
    }
}
