# atla

Unified Atlassian CLI for Jira and Confluence.

`atla` is a Rust CLI for day-to-day Atlassian Cloud work. It provides Jira and
Confluence commands with profile-based authentication, human-friendly tables,
and machine-friendly output formats.

## Documentation

| File | Description |
| --- | --- |
| [Getting Started](docs/getting-started.md) | Installation, first-time setup, shell completions, quick demo |
| [Authentication](docs/authentication.md) | Auth commands, multi-profile management, token storage, env vars |
| [Configuration](docs/configuration.md) | Config keys, aliases, config.toml schema, environment overrides |
| [Jira](docs/jira.md) | Full Jira command reference: projects, issues, boards, sprints |
| [Confluence](docs/confluence.md) | Full Confluence command reference: spaces, pages, blogs, attachments |
| [Output Formats](docs/output-formats.md) | Global flags, output formats, scripting/CI patterns |
| [Code Generation](docs/code-generation.md) | Progenitor-based API client generation, spec filtering, update workflow |
| [Agent Reference](docs/agent-reference.md) | Structured command reference for AI agents and automation |

## Install

Download a prebuilt binary from GitHub Releases, use the generated installer, or
install through mise after the first release is published.

Installer script:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NeoHsu/atla/releases/latest/download/atla-installer.sh | sh
```

Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/NeoHsu/atla/releases/latest/download/atla-installer.ps1 | iex"
```

Direct release downloads:

| Platform | Asset |
| --- | --- |
| Apple Silicon macOS | `atla-aarch64-apple-darwin.tar.xz` |
| Intel macOS | `atla-x86_64-apple-darwin.tar.xz` |
| ARM64 Linux | `atla-aarch64-unknown-linux-gnu.tar.xz` |
| x64 Linux | `atla-x86_64-unknown-linux-gnu.tar.xz` |
| x64 Windows | `atla-x86_64-pc-windows-msvc.zip` |

Each archive contains the prebuilt `atla` executable plus README, license, and
changelog files. Checksums are published next to the release assets.

mise:

```bash
mise use -g github:NeoHsu/atla
```

In `mise.toml`:

```toml
[tools]
"github:NeoHsu/atla" = "latest"
```

From source:

```bash
cargo install --git https://github.com/NeoHsu/atla atla
```

## Current Status

Phase 1 is ready for an initial binary release.

| Area | Status | Notes |
| --- | --- | --- |
| Workspace and CLI | Stable | Cargo workspace, command tree, global output/profile flags. |
| Authentication | Stable | Profile config in `~/.config/atla/config.toml`; tokens stored in OS keyring. |
| Jira commands | Stable | Projects, search, issues, comments, assignment, transitions, boards, sprints. |
| Confluence commands | Stable | Spaces, pages, blogs, search, attachments, page labels, page comments. |
| Generated clients | Stable | Jira v3, Confluence v2, and scoped Confluence v1 clients are checked in. |
| Release automation | Stable | GitHub Releases via cargo-dist. |
| Spec refresh automation | Stable | Scheduled workflow opens PRs for Atlassian OpenAPI spec updates. |

## Feature Matrix

| Product | Resource | Commands | Status |
| --- | --- | --- | --- |
| Core | Auth | `login`, `logout`, `status`, `switch` | Stable |
| Core | Config | `set`, `get`, `list` | Stable |
| Jira | Projects | `list`, `view` | Stable |
| Jira | Search | JQL search with table, JSON, CSV, and key output | Stable |
| Jira | Issues | `list`, `create`, `view`, `update`, `edit`, `delete` | Stable |
| Jira | Issue fields | Summary, description, arbitrary JSON fields, labels | Stable |
| Jira | Assignment | `assign --to me`, account ID, or user query | Stable |
| Jira | Transitions | List or apply transitions, with interactive selection when possible | Stable |
| Jira | Comments | `comment add`, `comment list` | Stable |
| Jira | Boards | `board list` with project, type, and name filters | Stable |
| Jira | Sprints | `sprint list`, `sprint active`, `sprint view` | Stable |
| Confluence | Spaces | `list`, `view`, `create`, `update`, `delete` | Stable |
| Confluence | Pages | `list`, `view`, `create`, `update`, `move`, `delete` | Stable |
| Confluence | Page content | Storage, wiki, Atlas Doc Format input; storage, ADF, markdown view output | Stable |
| Confluence | Page labels | `label list`, `label add`, `label remove` | Stable |
| Confluence | Page comments | `comment add`, `comment list` | Stable |
| Confluence | Blogs | `list`, `view`, `create`, `update`, `delete` | Stable |
| Confluence | Search | CQL search through scoped v1 REST endpoint | Stable |
| Confluence | Attachments | `list`, `view`, `upload`, `download`, `delete` | Stable |
| Output | Formats | `table`, `json`, `csv`, `keys` | Stable |
| Safety | Dry runs | Global `--dry-run` for mutating workflows | Stable |

## Configuration

`atla` stores profile configuration in `~/.config/atla/config.toml` by default.
API tokens are stored through the OS keyring and are not written to config files.

For isolated development runs, override the config path:

```bash
ATLA_CONFIG=/tmp/atla-config.toml cargo run -p atla -- config list
```

Initial auth:

```bash
atla auth login --instance https://example.atlassian.net --email you@example.com --token "$ATLASSIAN_TOKEN"
atla auth status
atla config set default-project PROJ
atla config set alias.mine "jira search 'assignee = currentUser() order by updated desc'"
atla config list --output json
```

`atla` stores API tokens in the OS keyring by default. In headless or container
environments where keyring access is unavailable, use an explicit file-backed
credential store or provide a token through the environment:

```bash
atla auth login --storage file --instance https://example.atlassian.net --email you@example.com --token "$ATLASSIAN_TOKEN"
ATLA_TOKEN="$ATLASSIAN_TOKEN" atla jira project list
```

File-backed credentials are stored separately from the main config in
`~/.config/atla/credentials.toml` by default. Override paths with `ATLA_CONFIG`
and `ATLA_CREDENTIALS` for isolated runs.

Aliases expand before command parsing, so the alias above can be used as:

```bash
atla mine --limit 25
```

Shell completions:

```bash
atla completion bash > ~/.local/share/bash-completion/completions/atla
atla completion zsh > ~/.zfunc/_atla
atla completion fish > ~/.config/fish/completions/atla.fish
```

## Usage

Jira examples:

```bash
atla jira project list
atla jira project list --query platform --limit 25 --output json
atla jira project view PROJ
atla jira search "project = PROJ order by created desc"
atla jira issue list --project PROJ --status "In Progress"
atla jira issue create --project PROJ --type Task --summary "Fix login"
atla jira issue update PROJ-123 --summary "Updated summary"
atla jira issue edit PROJ-123 --labels add:urgent,remove:low
atla jira issue view PROJ-123 --web
atla jira issue assign PROJ-123 --to me
atla jira issue transition PROJ-123 --to Done
atla jira issue comment add PROJ-123 "Ready for review"
atla jira issue delete PROJ-123 --yes
atla jira board list --project PROJ
atla jira sprint active --board 84
```

Confluence examples:

```bash
atla confluence space list
atla confluence space create "Development" --key DEV --description "Team docs"
atla confluence page list --space DEV
atla confluence page view 67890 --format markdown
atla confluence page create --space DEV --title "Meeting Notes" --body-file notes.html
atla confluence page update 67890 --title "Updated Notes"
atla confluence page move 67890 --parent 12345
atla confluence page label add 67890 runbook urgent
atla confluence page comment add 67890 "Looks good"
atla confluence blog create --space DEV --title "Release Notes" --body-file release.html
atla confluence search "type=page AND space=DEV"
atla confluence attachment upload 67890 ./diagram.png --comment "Updated diagram"
atla confluence attachment download 13579 --output ./diagram.png
```

Confluence v2 remains the primary generated client. Confluence search,
attachment upload, and page label mutation use scoped Confluence v1 REST
endpoints where v2 does not expose the required operation.

## Development

```bash
mise trust
mise install
cargo check
cargo test
```

Refresh Atlassian specs:

```bash
scripts/update-specs.sh
cargo check --workspace
```

## Release

Releases are built by cargo-dist. The release workflow runs on version tags such
as `v0.1.0`, builds Linux, macOS, and Windows binaries, creates a GitHub
Release with prebuilt binaries and installer scripts.

Release checklist:

```bash
cargo test --workspace
git tag v0.1.0
git push origin v0.1.0
```

Releases are built by cargo-dist and published to GitHub Releases automatically.
