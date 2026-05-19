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

    #[test]
    fn command_index_basic() {
        assert_eq!(command_index(&args(&["atla", "jira", "list"])), Some(1));
    }

    #[test]
    fn command_index_skips_dry_run_flag() {
        assert_eq!(
            command_index(&args(&["atla", "--dry-run", "jira"])),
            Some(2)
        );
    }

    #[test]
    fn command_index_skips_no_input_flag() {
        assert_eq!(
            command_index(&args(&["atla", "--no-input", "search"])),
            Some(2)
        );
    }

    #[test]
    fn command_index_skips_verbose_flag() {
        assert_eq!(
            command_index(&args(&["atla", "--verbose", "config"])),
            Some(2)
        );
    }

    #[test]
    fn command_index_skips_output_with_value() {
        assert_eq!(
            command_index(&args(&["atla", "--output", "json", "jira"])),
            Some(3)
        );
    }

    #[test]
    fn command_index_skips_short_output() {
        assert_eq!(
            command_index(&args(&["atla", "-o", "table", "jira"])),
            Some(3)
        );
    }

    #[test]
    fn command_index_skips_profile_with_value() {
        assert_eq!(
            command_index(&args(&["atla", "--profile", "work", "jira"])),
            Some(3)
        );
    }

    #[test]
    fn command_index_handles_output_equals() {
        assert_eq!(
            command_index(&args(&["atla", "--output=json", "jira"])),
            Some(2)
        );
    }

    #[test]
    fn command_index_handles_profile_equals() {
        assert_eq!(
            command_index(&args(&["atla", "--profile=work", "jira"])),
            Some(2)
        );
    }

    #[test]
    fn command_index_double_dash_separator() {
        assert_eq!(command_index(&args(&["atla", "--", "jira"])), Some(2));
    }

    #[test]
    fn command_index_double_dash_no_following_args() {
        assert_eq!(command_index(&args(&["atla", "--"])), None);
    }

    #[test]
    fn command_index_only_program_name() {
        assert_eq!(command_index(&args(&["atla"])), None);
    }

    #[test]
    fn command_index_unknown_flag_treated_as_command() {
        assert_eq!(command_index(&args(&["atla", "--unknown-flag"])), Some(1));
    }

    #[test]
    fn expands_alias_no_trailing_args() {
        let mut config = AtlaConfig::default();
        config
            .aliases
            .insert("ls".to_owned(), "jira project list".to_owned());

        let expanded = expand_args_with_config(args(&["atla", "ls"]), &config).expect("expand");

        assert_eq!(expanded, args(&["atla", "jira", "project", "list"]));
    }

    #[test]
    fn expands_alias_with_dry_run_before() {
        let mut config = AtlaConfig::default();
        config.aliases.insert(
            "mine".to_owned(),
            "jira search 'assignee = currentUser()'".to_owned(),
        );

        let expanded = expand_args_with_config(
            args(&["atla", "--dry-run", "mine", "--limit", "5"]),
            &config,
        )
        .expect("expand");

        assert_eq!(
            expanded,
            args(&[
                "atla",
                "--dry-run",
                "jira",
                "search",
                "assignee = currentUser()",
                "--limit",
                "5"
            ])
        );
    }

    #[test]
    fn expands_alias_with_no_input_before() {
        let mut config = AtlaConfig::default();
        config.aliases.insert(
            "mine".to_owned(),
            "jira search 'assignee = currentUser()'".to_owned(),
        );

        let expanded = expand_args_with_config(
            args(&["atla", "--no-input", "mine", "--limit", "5"]),
            &config,
        )
        .expect("expand");

        assert_eq!(
            expanded,
            args(&[
                "atla",
                "--no-input",
                "jira",
                "search",
                "assignee = currentUser()",
                "--limit",
                "5"
            ])
        );
    }

    #[test]
    fn expands_alias_with_output_equals() {
        let mut config = AtlaConfig::default();
        config.aliases.insert(
            "mine".to_owned(),
            "jira search 'assignee = currentUser()'".to_owned(),
        );

        let expanded = expand_args_with_config(args(&["atla", "--output=json", "mine"]), &config)
            .expect("expand");

        assert_eq!(
            expanded,
            args(&[
                "atla",
                "--output=json",
                "jira",
                "search",
                "assignee = currentUser()"
            ])
        );
    }

    #[test]
    fn expands_alias_with_profile_flag() {
        let mut config = AtlaConfig::default();
        config.aliases.insert(
            "mine".to_owned(),
            "jira search 'assignee = currentUser()'".to_owned(),
        );

        let expanded =
            expand_args_with_config(args(&["atla", "--profile", "work", "mine"]), &config)
                .expect("expand");

        assert_eq!(
            expanded,
            args(&[
                "atla",
                "--profile",
                "work",
                "jira",
                "search",
                "assignee = currentUser()"
            ])
        );
    }

    #[test]
    fn alias_to_alias_expands_chain() {
        let mut config = AtlaConfig::default();
        config
            .aliases
            .insert("ls".to_owned(), "project list".to_owned());
        config
            .aliases
            .insert("project".to_owned(), "jira project".to_owned());

        let expanded = expand_args_with_config(args(&["atla", "ls"]), &config).expect("expand");

        assert_eq!(expanded, args(&["atla", "jira", "project", "list"]));
    }

    #[test]
    fn alias_expansion_limit_error() {
        let mut config = AtlaConfig::default();
        // Circular chain: a1 → a2 → ... → a9 → a1
        for i in 1..=8 {
            config
                .aliases
                .insert(format!("a{i}"), format!("a{}", i + 1));
        }
        config.aliases.insert("a9".to_owned(), "a1".to_owned());

        let result = expand_args_with_config(args(&["atla", "a1"]), &config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("alias expansion exceeded 8 steps")
        );
    }

    #[test]
    fn empty_alias_expansion_errors() {
        let mut config = AtlaConfig::default();
        config.aliases.insert("empty".to_owned(), "".to_owned());

        let result = expand_args_with_config(args(&["atla", "empty"]), &config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expands to no arguments")
        );
    }

    #[test]
    fn no_args_returns_empty_or_program_only() {
        assert_eq!(command_index(&[]), None);
        assert_eq!(
            expand_args_with_config(vec![], &AtlaConfig::default()).expect("expand"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn command_index_multiple_flags_then_command() {
        assert_eq!(
            command_index(&args(&[
                "atla",
                "--verbose",
                "--dry-run",
                "--no-input",
                "jira"
            ])),
            Some(4)
        );
    }

    #[test]
    fn command_index_multiple_value_flags_then_command() {
        assert_eq!(
            command_index(&args(&[
                "atla",
                "--output",
                "json",
                "--profile",
                "work",
                "jira"
            ])),
            Some(5)
        );
    }
}
