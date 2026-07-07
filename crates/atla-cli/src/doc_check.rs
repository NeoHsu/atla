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
        if id == "help" || id == "version" {
            continue;
        }
        if path != "atla" && arg.is_global_set() {
            continue;
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
