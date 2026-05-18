use anyhow::Context;
use atla_core::{AtlaConfig, ConfigStore};

const MAX_ALIAS_EXPANSIONS: usize = 8;

pub fn expand_args(args: Vec<String>) -> anyhow::Result<Vec<String>> {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(args);
    }

    let store = ConfigStore::default_store().context("failed to find config location")?;
    let config = store.load().context("failed to load config")?;
    expand_args_with_config(args, &config)
}

fn expand_args_with_config(
    mut args: Vec<String>,
    config: &AtlaConfig,
) -> anyhow::Result<Vec<String>> {
    for _ in 0..MAX_ALIAS_EXPANSIONS {
        let Some(command_index) = command_index(&args) else {
            return Ok(args);
        };
        let Some(expansion) = config.aliases.get(&args[command_index]) else {
            return Ok(args);
        };

        let expanded = shell_words::split(expansion)
            .with_context(|| format!("failed to parse alias `{}`", args[command_index]))?;
        if expanded.is_empty() {
            anyhow::bail!("alias `{}` expands to no arguments", args[command_index]);
        }

        args.splice(command_index..=command_index, expanded);
    }

    anyhow::bail!("alias expansion exceeded {MAX_ALIAS_EXPANSIONS} steps")
}

fn command_index(args: &[String]) -> Option<usize> {
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--" => return (index + 1 < args.len()).then_some(index + 1),
            "--verbose" | "--dry-run" | "--no-input" => index += 1,
            "-o" | "--output" | "--profile" => index += 2,
            arg if arg.starts_with("--output=") || arg.starts_with("--profile=") => index += 1,
            arg if arg.starts_with('-') => return Some(index),
            _ => return Some(index),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn expands_first_command_alias_after_global_flags() {
        let mut config = AtlaConfig::default();
        config.aliases.insert(
            "mine".to_owned(),
            "jira search 'assignee = currentUser()'".to_owned(),
        );

        let expanded = expand_args_with_config(
            args(&["atla", "--output", "json", "mine", "--limit", "5"]),
            &config,
        )
        .expect("expand");

        assert_eq!(
            expanded,
            args(&[
                "atla",
                "--output",
                "json",
                "jira",
                "search",
                "assignee = currentUser()",
                "--limit",
                "5"
            ])
        );
    }

    #[test]
    fn leaves_unknown_aliases_unchanged() {
        let config = AtlaConfig::default();
        let original = args(&["atla", "jira", "project", "list"]);
        assert_eq!(
            expand_args_with_config(original.clone(), &config).expect("expand"),
            original
        );
    }
}
