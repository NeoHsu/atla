//! Doc-drift protection: every runnable `atla` example in current-contract
//! user, maintainer, spec, ADR, and skill Markdown must parse against the real
//! clap definition. Exact Jira/Confluence references and the full CLI surface
//! are also checked so interface changes fail CI until docs stay synchronized.

use std::path::{Path, PathBuf};

use clap::{CommandFactory, Parser};

use crate::cli::Cli;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn collect_markdown_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
}

fn doc_files() -> Vec<PathBuf> {
    let root = repo_root();
    let mut files = vec![
        root.join("README.md"),
        root.join("CLAUDE.md"),
        root.join("CONTRIBUTING.md"),
        root.join("specs/README.md"),
        root.join("specs/PATCHES.md"),
        root.join("specs/improvement-roadmap.md"),
    ];
    // Design proposals under specs/ may intentionally describe future CLI syntax.
    // Operational spec docs, maintainer docs, ADRs, and runtime references are contracts.
    for directory in ["docs", "skills/atla-cli"] {
        collect_markdown_files(&root.join(directory), &mut files);
    }
    files.sort();
    files.dedup();
    files
}

/// Quote-aware token splitter. Returns `None` for lines with unbalanced quotes.
fn shell_lex(line: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut token_open = false;
    let mut in_single = false;
    let mut in_double = false;
    for c in line.chars() {
        match c {
            '\'' if !in_double => {
                in_single = !in_single;
                token_open = true;
            }
            '"' if !in_single => {
                in_double = !in_double;
                token_open = true;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if token_open {
                    tokens.push(std::mem::take(&mut current));
                    token_open = false;
                }
            }
            '#' if !in_single && !in_double && !token_open => break,
            _ => {
                current.push(c);
                token_open = true;
            }
        }
    }
    if in_single || in_double {
        return None;
    }
    if token_open {
        tokens.push(current);
    }
    Some(tokens)
}

const GLOBAL_VALUE_FLAGS: &[&str] = &[
    "--output",
    "-o",
    "--profile",
    "--max-pages",
    "--max-items",
    "--max-bytes",
    "--timeout",
];
const TOP_COMMANDS: &[&str] = &[
    "auth",
    "config",
    "jira",
    "confluence",
    "doctor",
    "explain-policy",
    "operation",
    "schema",
    "plan",
    "apply",
    "completion",
];

/// Normalize a candidate example into argv for clap, or `None` if the line is
/// not a self-contained runnable `atla` invocation.
fn extract_argv(line: &str) -> Option<Vec<String>> {
    let line = line.trim().trim_start_matches("$ ").trim();
    if line.contains('|') || line.contains("$(") || line.contains('`') {
        return None;
    }
    let mut tokens = shell_lex(line)?;
    // Drop leading VAR=value environment assignments.
    while tokens.first().is_some_and(|t| {
        !t.starts_with('-')
            && t.split('=').next().is_some_and(|name| {
                !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            })
            && t.contains('=')
    }) {
        tokens.remove(0);
    }
    if tokens.first().map(String::as_str) != Some("atla") {
        return None;
    }
    // Truncate at shell redirections.
    if let Some(pos) = tokens.iter().position(|t| t.starts_with('>')) {
        tokens.truncate(pos);
    }
    // An `...` ellipsis marks an abbreviated illustration, not a runnable example.
    if tokens.iter().any(|t| t == "...") {
        return None;
    }
    // Alternative notation like `login/logout/status` or `bash/zsh` lists
    // choices; file paths and URLs keep a '.', '~', or leading '/' marker.
    if tokens.iter().any(|t| {
        t.contains('/')
            && !t.starts_with(['/', '.', '~'])
            && t.split('/').all(|part| !part.contains('.'))
    }) {
        return None;
    }
    // Placeholder handling: `<TOKEN>` becomes a dummy value; other unquoted
    // placeholder syntax (`[--flag]`, bare `<X`) marks a usage summary, not a
    // runnable example.
    for token in tokens.iter_mut() {
        if token.starts_with('<') && token.ends_with('>') {
            *token = "PLACEHOLDER".to_string();
        } else if !token.contains(' ') && (token.contains('<') || token.contains('[')) {
            return None;
        }
    }
    // The first non-global-flag token must be a real subcommand; this skips
    // user-alias examples like `atla mine`.
    let mut rest = tokens[1..].iter();
    let subcommand = loop {
        let token = rest.next()?;
        if GLOBAL_VALUE_FLAGS.contains(&token.as_str()) {
            rest.next()?;
        } else if !token.starts_with('-') {
            break token.clone();
        }
    };
    if !TOP_COMMANDS.contains(&subcommand.as_str()) {
        return None;
    }
    // A bare group mention like `atla jira` is prose, not an example.
    if tokens.last() == Some(&subcommand) {
        return None;
    }
    Some(tokens)
}

/// Inline prose may name a command without all required args, while runnable
/// fenced examples must satisfy clap's required arguments. Placeholders can
/// still fail typed value validation after proving that their flags exist.
fn is_acceptable_error(err: &clap::Error, allow_missing_required: bool) -> bool {
    use clap::error::ErrorKind;
    match err.kind() {
        ErrorKind::MissingRequiredArgument if allow_missing_required => true,
        ErrorKind::MissingSubcommand
        | ErrorKind::DisplayHelp
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => true,
        ErrorKind::InvalidValue | ErrorKind::ValueValidation => {
            err.to_string().contains("PLACEHOLDER")
        }
        _ => false,
    }
}

/// Collect candidate example lines from one markdown file: lines inside
/// ```bash fences (with backslash continuations joined) plus inline
/// `atla ...` code spans. The boolean marks a runnable fenced command; inline
/// command-path mentions may intentionally omit required values.
fn candidate_lines(content: &str) -> Vec<(usize, String, bool)> {
    let mut candidates = Vec::new();
    let mut in_bash = false;
    let mut pending: Option<(usize, String)> = None;
    for (idx, raw) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if trimmed.starts_with("```") {
            in_bash = !in_bash && trimmed.trim_start_matches('`').trim() == "bash";
            pending = None;
            continue;
        }
        if in_bash {
            let (start, mut joined) = pending.take().unwrap_or((line_no, String::new()));
            if !joined.is_empty() {
                joined.push(' ');
            }
            joined.push_str(trimmed.trim_end_matches('\\').trim());
            if trimmed.ends_with('\\') {
                pending = Some((start, joined));
            } else {
                candidates.push((start, joined, true));
            }
        } else {
            for span in raw.split('`').skip(1).step_by(2) {
                if span.starts_with("atla ") {
                    candidates.push((line_no, span.to_string(), false));
                }
            }
        }
    }
    candidates
}

/// Collect logical `atla ...` commands from every fenced block, including the
/// untagged usage summaries that are intentionally not parsed as shell.
fn fenced_commands(content: &str) -> Vec<(usize, String)> {
    let mut commands = Vec::new();
    let mut in_fence = false;
    let mut pending: Option<(usize, String)> = None;
    let mut explicit_continuation = false;

    for (idx, raw) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if trimmed.starts_with("```") {
            if let Some(command) = pending.take() {
                commands.push(command);
            }
            in_fence = !in_fence;
            explicit_continuation = false;
            continue;
        }
        if !in_fence {
            continue;
        }
        if trimmed.starts_with("atla ") {
            if let Some(command) = pending.take() {
                commands.push(command);
            }
            explicit_continuation = trimmed.ends_with('\\');
            pending = Some((line_no, trimmed.trim_end_matches('\\').trim().to_owned()));
            continue;
        }
        let usage_continuation = trimmed.starts_with("[--")
            || trimmed.starts_with("[-")
            || trimmed.starts_with("--")
            || trimmed.starts_with("-s ");
        if !trimmed.is_empty()
            && (explicit_continuation || usage_continuation)
            && let Some((_, command)) = pending.as_mut()
        {
            command.push(' ');
            command.push_str(trimmed.trim_end_matches('\\').trim());
            explicit_continuation = trimmed.ends_with('\\');
            continue;
        }
        if let Some(command) = pending.take() {
            commands.push(command);
            explicit_continuation = false;
        }
    }
    if let Some(command) = pending {
        commands.push(command);
    }
    commands
}

fn documented_command_and_path<'a>(
    root: &'a clap::Command,
    line: &str,
) -> Option<(&'a clap::Command, String)> {
    let tokens = shell_lex(line)?;
    if tokens.first().map(String::as_str) != Some("atla")
        || tokens.get(1).is_none_or(|token| token.starts_with('-'))
    {
        return None;
    }

    let mut command = root;
    let mut path = vec![root.get_name().to_owned()];
    for token in &tokens[1..] {
        let token = token.trim_matches(|c: char| matches!(c, '[' | ']' | '(' | ')' | '<' | '>'));
        let Some(subcommand) = command.get_subcommands().find(|subcommand| {
            subcommand.get_name() == token
                || subcommand.get_all_aliases().any(|alias| alias == token)
        }) else {
            break;
        };
        path.push(subcommand.get_name().to_owned());
        command = subcommand;
    }
    (path.len() > 1).then(|| (command, path.join(" ")))
}

fn documented_command<'a>(root: &'a clap::Command, line: &str) -> Option<&'a clap::Command> {
    documented_command_and_path(root, line).map(|(command, _)| command)
}

fn documented_command_path(root: &clap::Command, line: &str) -> Option<String> {
    documented_command_and_path(root, line).map(|(_, path)| path)
}

fn leaf_command_paths(
    command: &clap::Command,
    path: &str,
    paths: &mut std::collections::BTreeSet<String>,
) {
    let mut has_subcommands = false;
    for subcommand in command
        .get_subcommands()
        .filter(|subcommand| subcommand.get_name() != "help")
    {
        has_subcommands = true;
        let subcommand_path = format!("{path} {}", subcommand.get_name());
        leaf_command_paths(subcommand, &subcommand_path, paths);
    }
    if !has_subcommands {
        paths.insert(path.to_owned());
    }
}

fn product_leaf_command_paths(
    root: &clap::Command,
    product: &str,
) -> std::collections::BTreeSet<String> {
    let mut paths = std::collections::BTreeSet::new();
    if let Some(command) = root
        .get_subcommands()
        .find(|command| command.get_name() == product)
    {
        leaf_command_paths(command, &format!("atla {product}"), &mut paths);
    }
    paths
}

fn documented_flags(line: &str) -> Vec<String> {
    // Usage alternatives put another option after `|`; a shell pipeline does not.
    let shell_pipeline = line.contains(" | ") && !line.contains(" | -");
    let line = if shell_pipeline {
        line.split_once(" | ").map_or(line, |(command, _)| command)
    } else {
        line
    };
    shell_lex(line)
        .unwrap_or_default()
        .into_iter()
        .flat_map(|token| {
            let token = token
                .trim_matches(|c: char| matches!(c, '[' | ']' | '(' | ')' | '<' | '>' | ',' | '|'));
            token
                .split('/')
                .filter_map(|part| {
                    if part.starts_with("--") {
                        Some(
                            part.chars()
                                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                                .collect(),
                        )
                    } else if part.starts_with('-')
                        && part.len() == 2
                        && part.chars().nth(1).is_some_and(|c| c.is_ascii_alphabetic())
                    {
                        Some(part.to_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn command_flags(command: &clap::Command) -> std::collections::BTreeSet<String> {
    let mut flags = std::collections::BTreeSet::new();
    for arg in command.get_arguments() {
        if let Some(long) = arg.get_long() {
            flags.insert(format!("--{long}"));
        }
        if let Some(aliases) = arg.get_all_aliases() {
            flags.extend(aliases.into_iter().map(|alias| format!("--{alias}")));
        }
        if let Some(short) = arg.get_short() {
            flags.insert(format!("-{short}"));
        }
        if let Some(aliases) = arg.get_all_short_aliases() {
            flags.extend(aliases.into_iter().map(|alias| format!("-{alias}")));
        }
    }
    flags
}

fn unknown_flags(command: &clap::Command, summary: &str) -> Vec<String> {
    let allowed = command_flags(command);
    documented_flags(summary)
        .into_iter()
        .filter(|flag| !allowed.contains(flag))
        .collect()
}

fn missing_local_flags(command: &clap::Command, summary: &str) -> Vec<String> {
    let documented: std::collections::BTreeSet<_> = documented_flags(summary).into_iter().collect();
    command
        .get_arguments()
        .filter(|arg| {
            !arg.is_positional()
                && !arg.is_global_set()
                && !matches!(arg.get_id().as_str(), "help" | "version")
        })
        .filter_map(|arg| {
            let long = arg.get_long().map(|flag| format!("--{flag}"));
            let short = arg.get_short().map(|flag| format!("-{flag}"));
            let present = long.as_ref().is_some_and(|flag| documented.contains(flag))
                || short.as_ref().is_some_and(|flag| documented.contains(flag));
            (!present).then_some(long.or(short)).flatten()
        })
        .collect()
}

fn syntax_summaries(content: &str) -> Vec<(usize, String)> {
    let lines: Vec<_> = content.lines().collect();
    let mut summaries = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if lines[index].trim() != "**Syntax**" {
            index += 1;
            continue;
        }
        index += 1;
        while index < lines.len() && !lines[index].trim().starts_with("```") {
            index += 1;
        }
        if index == lines.len() {
            break;
        }
        index += 1;
        let line_number = index + 1;
        let mut summary = String::new();
        while index < lines.len() && !lines[index].trim().starts_with("```") {
            let fragment = lines[index].trim().trim_end_matches('\\').trim();
            if !fragment.is_empty() {
                if !summary.is_empty() {
                    summary.push(' ');
                }
                summary.push_str(fragment);
            }
            index += 1;
        }
        if summary.starts_with("atla ") {
            summaries.push((line_number, summary));
        }
    }
    summaries
}

fn command_table_rows(content: &str) -> Vec<(usize, String)> {
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if !line.starts_with("| `") {
                return None;
            }
            let cells: Vec<_> = line.trim_matches('|').split('|').map(str::trim).collect();
            if cells.len() < 3 {
                return None;
            }
            let command = cells[0].replace('`', "");
            let flags = cells[2].replace('`', "");
            Some((index + 1, format!("atla {command} {flags}")))
        })
        .collect()
}

#[test]
fn cli_definition_is_consistent() {
    Cli::command().debug_assert();
}

#[test]
fn doc_examples_parse() {
    let mut failures = Vec::new();
    let mut checked = 0usize;
    for file in doc_files() {
        let content = std::fs::read_to_string(&file).expect("read doc file");
        for (line_no, line, runnable) in candidate_lines(&content) {
            let Some(argv) = extract_argv(&line) else {
                continue;
            };
            checked += 1;
            if let Err(err) = Cli::try_parse_from(&argv) {
                if is_acceptable_error(&err, !runnable) {
                    continue;
                }
                failures.push(format!(
                    "{}:{}: `{}`\n    {}",
                    file.display(),
                    line_no,
                    line,
                    err.to_string().lines().next().unwrap_or_default(),
                ));
            }
        }
    }
    assert!(
        checked > 50,
        "doc example extraction broke: only {checked} examples found"
    );
    assert!(
        failures.is_empty(),
        "{} documented atla example(s) no longer parse against the CLI.\n\
         Fix the docs/skill files or the CLI definition:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn documented_fenced_flags_match_cli() {
    let mut cli = Cli::command();
    cli.build();
    let mut checked = 0usize;
    let mut failures = Vec::new();

    for file in doc_files() {
        let content = std::fs::read_to_string(&file).expect("read doc file");
        for (line_no, line) in fenced_commands(&content) {
            let Some(command) = documented_command(&cli, &line) else {
                continue;
            };
            let allowed = command_flags(command);
            for flag in documented_flags(&line) {
                checked += 1;
                if !allowed.contains(&flag) {
                    failures.push(format!(
                        "{}:{line_no}: `{flag}` is not accepted by `{}`\n    {line}",
                        file.display(),
                        command.get_name()
                    ));
                }
            }
        }
    }

    assert!(
        checked > 100,
        "fenced usage extraction broke: only {checked} flags found"
    );
    assert!(
        failures.is_empty(),
        "{} documented fenced flag(s) no longer exist in clap:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn exact_reference_summaries_match_cli() {
    let mut cli = Cli::command();
    cli.build();
    let root = repo_root();
    let mut checked = 0usize;
    let mut failures = Vec::new();

    for (product, relative) in [
        ("jira", "docs/jira.md"),
        ("confluence", "docs/confluence.md"),
    ] {
        let file = root.join(relative);
        let content = std::fs::read_to_string(&file).expect("read topic reference");
        let mut documented_paths = std::collections::BTreeSet::new();
        for (line_number, summary) in syntax_summaries(&content) {
            let Some(command) = documented_command(&cli, &summary) else {
                continue;
            };
            checked += 1;
            if let Some(path) = documented_command_path(&cli, &summary) {
                documented_paths.insert(path);
            }
            let missing = missing_local_flags(command, &summary);
            if !missing.is_empty() {
                failures.push(format!(
                    "{}:{line_number}: exact syntax omits {}\n    {summary}",
                    file.display(),
                    missing.join(", ")
                ));
            }
            let unknown = unknown_flags(command, &summary);
            if !unknown.is_empty() {
                failures.push(format!(
                    "{}:{line_number}: exact syntax contains unknown flags {}\n    {summary}",
                    file.display(),
                    unknown.join(", ")
                ));
            }
        }
        for missing in product_leaf_command_paths(&cli, product).difference(&documented_paths) {
            failures.push(format!(
                "{}: exact syntax section is missing `{missing}`",
                file.display()
            ));
        }
    }

    let agent_reference = root.join("docs/agent-reference.md");
    let content = std::fs::read_to_string(&agent_reference).expect("read agent reference");
    let mut documented_paths = std::collections::BTreeSet::new();
    for (line_number, summary) in command_table_rows(&content) {
        let Some(command) = documented_command(&cli, &summary) else {
            continue;
        };
        checked += 1;
        if let Some(path) = documented_command_path(&cli, &summary) {
            documented_paths.insert(path);
        }
        let missing = missing_local_flags(command, &summary);
        if !missing.is_empty() {
            failures.push(format!(
                "{}:{line_number}: command table omits {}\n    {summary}",
                agent_reference.display(),
                missing.join(", ")
            ));
        }
        let unknown = unknown_flags(command, &summary);
        if !unknown.is_empty() {
            failures.push(format!(
                "{}:{line_number}: command table contains unknown flags {}\n    {summary}",
                agent_reference.display(),
                unknown.join(", ")
            ));
        }
    }
    let expected_agent_paths = product_leaf_command_paths(&cli, "jira")
        .into_iter()
        .chain(product_leaf_command_paths(&cli, "confluence"))
        .collect::<std::collections::BTreeSet<_>>();
    for missing in expected_agent_paths.difference(&documented_paths) {
        failures.push(format!(
            "{}: command table is missing `{missing}`",
            agent_reference.display()
        ));
    }

    assert!(
        checked > 100,
        "exact reference extraction broke: only {checked} commands found"
    );
    assert!(
        failures.is_empty(),
        "{} exact reference entries omit CLI commands or flags:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn render_surface(cmd: &clap::Command, path: &str, out: &mut String) {
    let mut positionals = Vec::new();
    let mut flags = Vec::new();
    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        if id == "help" || (id == "version" && path == "atla") {
            continue;
        }
        if path != "atla" && arg.is_global_set() {
            let declared_by_plan = path == "atla plan" && matches!(id, "out" | "expires_in");
            if !declared_by_plan {
                continue;
            }
        }
        if arg.is_positional() {
            positionals.push(format!("<{}>", id.to_uppercase()));
        } else {
            let mut flag = String::new();
            if let Some(long) = arg.get_long() {
                flag.push_str("--");
                flag.push_str(long);
            }
            if let Some(short) = arg.get_short() {
                if !flag.is_empty() {
                    flag.push('/');
                }
                flag.push('-');
                flag.push(short);
            }
            flags.push(flag);
        }
    }
    flags.sort();
    let mut line = path.to_string();
    for positional in &positionals {
        line.push(' ');
        line.push_str(positional);
    }
    for flag in &flags {
        line.push(' ');
        line.push_str(flag);
    }
    let aliases: Vec<_> = cmd.get_all_aliases().collect();
    if !aliases.is_empty() {
        line.push_str(&format!("  (aliases: {})", aliases.join(", ")));
    }
    out.push_str(&line);
    out.push('\n');
    for sub in cmd.get_subcommands() {
        if sub.get_name() == "help" {
            continue;
        }
        render_surface(sub, &format!("{path} {}", sub.get_name()), out);
    }
}

fn validate_schema_fixture(schema: &serde_json::Value, value: &serde_json::Value, path: &str) {
    if let Some(expected) = schema.get("const") {
        assert_eq!(value, expected, "{path}: const mismatch");
    }
    if let Some(values) = schema.get("enum").and_then(serde_json::Value::as_array) {
        assert!(values.contains(value), "{path}: value is not in enum");
    }
    if let Some(types) = schema.get("type") {
        let matches_type = |kind: &str| match kind {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => true,
        };
        let valid = types.as_str().is_some_and(matches_type)
            || types.as_array().is_some_and(|types| {
                types
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .any(matches_type)
            });
        assert!(valid, "{path}: JSON type does not match schema");
    }
    if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array) {
        let object = value.as_object().expect("required applies to an object");
        for key in required.iter().filter_map(serde_json::Value::as_str) {
            assert!(object.contains_key(key), "{path}: missing required `{key}`");
        }
    }
    if let (Some(properties), Some(object)) = (
        schema
            .get("properties")
            .and_then(serde_json::Value::as_object),
        value.as_object(),
    ) {
        for (key, property_schema) in properties {
            if let Some(property) = object.get(key) {
                validate_schema_fixture(property_schema, property, &format!("{path}.{key}"));
            }
        }
    }
    if let (Some(item_schema), Some(items)) = (schema.get("items"), value.as_array()) {
        for (index, item) in items.iter().enumerate() {
            validate_schema_fixture(item_schema, item, &format!("{path}[{index}]"));
        }
    }
    if let (Some(min_items), Some(items)) = (
        schema.get("minItems").and_then(serde_json::Value::as_u64),
        value.as_array(),
    ) {
        assert!(items.len() as u64 >= min_items, "{path}: too few items");
    }
}

#[test]
fn json_contract_fixtures_match_published_schemas() {
    let root = repo_root().join("docs/schemas");
    let mut names = std::fs::read_dir(&root)
        .expect("read JSON schema directory")
        .flatten()
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_suffix(".schema.json"))
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    names.sort();
    assert!(!names.is_empty(), "no published JSON schemas found");
    let mut fixture_names = std::fs::read_dir(root.join("fixtures"))
        .expect("read JSON fixture directory")
        .flatten()
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_suffix(".json"))
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    fixture_names.sort();
    assert_eq!(names, fixture_names, "schema/fixture registry drift");
    for name in names {
        let schema: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(format!("{name}.schema.json")))
                .expect("read JSON schema"),
        )
        .expect("parse JSON schema");
        let fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures").join(format!("{name}.json")))
                .expect("read JSON fixture"),
        )
        .expect("parse JSON fixture");
        if let Some(all_of) = schema.get("allOf").and_then(serde_json::Value::as_array) {
            for referenced in all_of {
                if let Some(reference) = referenced.get("$ref").and_then(serde_json::Value::as_str)
                {
                    let referenced_schema: serde_json::Value = serde_json::from_str(
                        &std::fs::read_to_string(root.join(reference))
                            .expect("read referenced JSON schema"),
                    )
                    .expect("parse referenced JSON schema");
                    validate_schema_fixture(&referenced_schema, &fixture, &name);
                } else {
                    validate_schema_fixture(referenced, &fixture, &name);
                }
            }
        }
        validate_schema_fixture(&schema, &fixture, &name);
    }
}

#[test]
fn cli_surface_snapshot() {
    let mut cmd = Cli::command();
    cmd.build();
    let mut actual = String::from(
        "# Generated by `UPDATE_CLI_SURFACE=1 cargo test -p atla cli_surface`.\n\
         # One line per command: positionals, then flags. Do not edit by hand.\n",
    );
    render_surface(&cmd, "atla", &mut actual);

    let snapshot_path = repo_root().join("docs/cli-surface.txt");
    if std::env::var_os("UPDATE_CLI_SURFACE").is_some() {
        std::fs::write(&snapshot_path, &actual).expect("write cli surface snapshot");
        return;
    }
    let expected = std::fs::read_to_string(&snapshot_path).unwrap_or_default();
    assert!(
        expected == actual,
        "CLI surface changed but docs/cli-surface.txt was not regenerated.\n\
         1. Run: UPDATE_CLI_SURFACE=1 cargo test -p atla cli_surface\n\
         2. Update docs/ and skills/atla-cli/ for the changed commands\n\
            (see the 'Changing the CLI surface' checklist in CLAUDE.md).\n\
         Diff hint: compare docs/cli-surface.txt against the test output."
    );
}
