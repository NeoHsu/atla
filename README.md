# atla

Unified Atlassian CLI for Jira and Confluence.

`atla` is planned as a Rust CLI with:

- Jira Cloud and Confluence Cloud support.
- API clients generated from Atlassian OpenAPI specs.
- Human-friendly table output and machine-friendly JSON/CSV/keys output.
- Profile-based authentication with tokens stored outside config files.

## Current Status

This repository is in Phase 1 setup:

- Cargo workspace skeleton.
- CLI command tree.
- Core module boundaries for auth, profile, client, and markdown conversion.
- Generated API crates reserved for OpenAPI generator output.

See `docs/project-atla/adr/init_atla_cli.md` in the planning workspace for the full ADR.

## Development

```bash
mise trust
mise install
cargo check
cargo test
```

## Configuration

`atla` stores profile configuration in `~/.config/atla/config.toml` by default.
API tokens are stored through the OS keyring and are not written to the config file.

For isolated development runs, override the config path:

```bash
ATLA_CONFIG=/tmp/atla-config.toml cargo run -p atla -- config list
```

Initial auth commands:

```bash
atla auth login --instance https://example.atlassian.net --email you@example.com --token "$ATLASSIAN_TOKEN"
atla auth status
atla config set default-project PROJ
atla config list --output json
```

First Jira command:

```bash
atla jira project list
atla jira project list --query platform --limit 25 --output json
atla jira project list --output keys
atla jira project view PROJ
atla jira project view PROJ --output json
atla jira search "project = PROJ order by created desc"
atla jira search "assignee = currentUser() and status != Done" --limit 25 --output keys
atla jira issue view PROJ-123
atla jira issue view PROJ-123 --output json
```

First Confluence commands:

```bash
atla confluence space list
atla confluence space list --key DEV --output json
atla confluence space view DEV
atla confluence space view DEV --output json
atla confluence page list --space DEV
atla confluence page list --space-id 12345 --title "Meeting Notes"
atla confluence page view 67890
atla confluence page view 67890 --output json
```
