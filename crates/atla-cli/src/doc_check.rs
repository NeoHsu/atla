//! Doc-drift protection: every runnable `atla` example in docs/ and skills/
//! must parse against the real clap definition, and the full CLI surface is
//! snapshotted to docs/cli-surface.txt so interface changes fail CI until the
//! docs and skill references are updated (see CLAUDE.md checklist).

use std::path::{Path, PathBuf};

use clap::{CommandFactory, Parser};

use crate::cli::Cli;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn doc_files() -> Vec<PathBuf> {
    let root = repo_root();
    let mut files = vec![root.join("README.md")];
    for dir in ["docs", "skills/atla-cli", "skills/atla-cli/references"] {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
    }
    files.sort();
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

const GLOBAL_VALUE_FLAGS: &[&str] = &["--output", "-o", "--profile"];
const TOP_COMMANDS: &[&str] = &["auth", "config", "jira", "confluence", "completion"];

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

/// Prose often names a command without its required args, and placeholders
/// cannot satisfy typed values; both still prove the command path and flags
/// exist, which is all this check asserts.
fn is_acceptable_error(err: &clap::Error) -> bool {
    use clap::error::ErrorKind;
    match err.kind() {
        ErrorKind::MissingRequiredArgument
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
/// `atla ...` code spans.
fn candidate_lines(content: &str) -> Vec<(usize, String)> {
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
                candidates.push((start, joined));
            }
        } else {
            for span in raw.split('`').skip(1).step_by(2) {
                if span.starts_with("atla ") {
                    candidates.push((line_no, span.to_string()));
                }
            }
        }
    }
    candidates
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
        for (line_no, line) in candidate_lines(&content) {
            let Some(argv) = extract_argv(&line) else {
                continue;
            };
            checked += 1;
            if let Err(err) = Cli::try_parse_from(&argv) {
                if is_acceptable_error(&err) {
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
    for name in [
        "error-v1",
        "list-v1",
        "jira-issue-list-v1",
        "confluence-page-list-v1",
        "operation-plan-v1",
        "mutation-receipt-v1",
    ] {
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
                    validate_schema_fixture(&referenced_schema, &fixture, name);
                } else {
                    validate_schema_fixture(referenced, &fixture, name);
                }
            }
        }
        validate_schema_fixture(&schema, &fixture, name);
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
