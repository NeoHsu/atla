# atla

Unified Atlassian CLI for Jira and Confluence.

`atla` is a Rust single-binary CLI for day-to-day Atlassian Cloud work. It
provides Jira and Confluence commands with profile-based authentication,
human-friendly tables, and machine-friendly output formats.

## Why atla?

- Work with Jira issues, boards, and sprints without leaving the terminal.
- Read and update Confluence spaces, pages, blogs, labels, comments, and
  attachments from scripts or interactive workflows.
- Automate reporting and CI with stable `table`, `json`, `csv`, and `keys` output,
  mutation policies (`--dry-run`, `--read-only`), and context budgets.

## Install

Verified installer script (macOS/Linux):

```bash
base=https://github.com/NeoHsu/atla/releases/latest/download
curl --proto '=https' --tlsv1.2 -LsSfO "$base/atla-installer.sh"
curl --proto '=https' --tlsv1.2 -LsSfO "$base/atla-installer.sh.sha256"
shasum -a 256 -c atla-installer.sh.sha256
sh atla-installer.sh
```

Verified Windows PowerShell installer:

```powershell
$base = "https://github.com/NeoHsu/atla/releases/latest/download"
Invoke-WebRequest "$base/atla-installer.ps1" -OutFile atla-installer.ps1
Invoke-WebRequest "$base/atla-installer.ps1.sha256" -OutFile atla-installer.ps1.sha256
$expected = ((Get-Content -Raw atla-installer.ps1.sha256).Trim() -split '\s+')[0]
$actual = (Get-FileHash -Algorithm SHA256 atla-installer.ps1).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "installer checksum verification failed" }
& .\atla-installer.ps1
```

Direct release downloads:

| Platform | Asset |
| --- | --- |
| Apple Silicon macOS | `atla-aarch64-apple-darwin.tar.xz` |
| Intel macOS | `atla-x86_64-apple-darwin.tar.xz` |
| ARM64 Linux | `atla-aarch64-unknown-linux-gnu.tar.xz` |
| x64 Linux | `atla-x86_64-unknown-linux-gnu.tar.xz` |
| x64 Windows | `atla-x86_64-pc-windows-msvc.zip` |

Each archive contains the prebuilt `atla` executable plus README and license files.
Checksums, build-provenance attestations, and a CycloneDX 1.5 binary SBOM (`atla.cdx.xml` plus
`atla.cdx.xml.sha256`) are published with the release assets; release notes come from the changelog.

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

## Common workflows

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --instance https://example.atlassian.net \
  --email you@example.com --token-stdin
atla jira search "assignee = currentUser() order by updated desc" --limit 10
atla jira issue transition PROJ-123 --to Done --dry-run
atla plan jira issue create --project PROJ --type Task --summary 'Agent task' --out issue-plan.json
atla apply issue-plan.json --yes --output json
atla confluence page view 67890 --format markdown
atla confluence search "type=page AND space=DEV" --output json
```

## Documentation

| File | Description |
| --- | --- |
| [Getting Started](docs/getting-started.md) | Installation, first-time setup, shell completions, quick demo |
| [Authentication](docs/authentication.md) | Auth commands, multi-profile management, token storage, env vars |
| [Configuration](docs/configuration.md) | Config keys, aliases, config.toml schema, environment overrides |
| [Jira](docs/jira.md) | Full Jira command reference: projects, issues, boards, sprints |
| [Confluence](docs/confluence.md) | Full Confluence command reference: spaces, pages, blogs, attachments |
| [Output Formats](docs/output-formats.md) | Global flags, output formats, scripting/CI patterns |
| [JSON Contracts](docs/json-schemas.md) | Versioned schemas, plans, receipts, compatibility policy |
| [Saved Plans](docs/plans.md) | Generate and safely apply expiring mutation plans |
| [Operation Policy](docs/policy.md) | Read-only enforcement, allow/deny rules, context budgets |
| [Config Migration](docs/migration-v1.md) | Schema v2 migration, backups, and rollback |
| [Compatibility](docs/compatibility.md) | Supported platforms and v1 stability policy |
| [Release Procedure](docs/releasing.md) | Artifacts, SBOM, provenance, and workflow hardening |
| [Live Sandbox Smoke Testing](docs/live-smoke.md) | Bounded Jira/Confluence contract coverage and cleanup ledger |
| [Contributing](CONTRIBUTING.md) | Development checks, CLI contracts, and PR process |
| [Code Generation](docs/code-generation.md) | Progenitor-based API client generation, spec filtering, update workflow |
| [Agent Reference](docs/agent-reference.md) | Structured command reference for AI agents and automation |

## atla-cli Skill

Install the bundled AI agent skill to enable `atla`-aware assistance — JQL/CQL
patterns, all command flags, scripting idioms, and safety rules.

From the published repository:

```bash
npx skills add NeoHsu/atla --skill atla-cli
```

From a local checkout of this repo, install the internal `skills/atla-cli` package with:

```bash
npx skills add . --skill atla-cli
```

For non-interactive setup across all supported agents, add `--agent '*' -y`:

```bash
npx skills add . --skill atla-cli --agent '*' -y
```

Use `--copy` if you want the installed skill to be a standalone copy instead of a symlink
back to the repo checkout.

## Feature Matrix

| Product | Resource | Commands |
| --- | --- | --- |
| Core | Auth | `login`, `logout`, `status`, `switch` |
| Core | Config | `set`, `get`, `list` |
| Jira | Projects | `list`, `view`, `issue-types` |
| Jira | Search | JQL search with table, JSON, CSV, and key output |
| Jira | Issues | `list`, `create`, `view`, `update`, `edit`, `delete` |
| Jira | Issue fields | `fields` — list create-meta fields with required flag, type, and allowed values |
| Jira | Assignment | `assign --to me`, account ID, or user query |
| Jira | Transitions | List or apply transitions, with interactive selection when possible |
| Jira | Comments | `comment add`, `comment list`, `comment update`, `comment delete` |
| Jira | Attachments | `attachment list`, `attachment upload`, `attachment download`, `attachment delete` |
| Jira | Links | `link add`, `link list`, `link remove`, `link github-links`, `link github-commits` |
| Jira | Worklogs | `worklog add`, `worklog list` |
| Jira | Boards | `board list` with project, type, and name filters; `board view` |
| Jira | Sprints | `sprint list`, `sprint active`, `sprint view`, `sprint create`, `sprint start`, `sprint close`, `sprint add`, `sprint remove`, `sprint issues` |
| Confluence | Spaces | `list`, `view`, `create`, `update`, `delete` |
| Confluence | Pages | `list`, `view`, `create`, `update`, `move`, `delete`, `children`, `copy` |
| Confluence | Page content | Storage, wiki, Atlas Doc Format input; storage, ADF, markdown view output |
| Confluence | Page labels | `label list`, `label add`, `label remove` |
| Confluence | Page comments | `comment add`, `comment list`, `comment delete` |
| Confluence | Blogs | `list`, `view`, `create`, `update`, `delete` |
| Confluence | Blog labels | `label list`, `label add`, `label remove` |
| Confluence | Blog comments | `comment add`, `comment list`, `comment delete` |
| Confluence | Search | CQL search through scoped v1 REST endpoint |
| Confluence | Attachments | `list`, `view`, `upload`, `download`, `delete` |
| Output | Formats | `table`, `json`, `csv`, `keys` |
| Safety | Mutation policy | Global `--dry-run` previews and `--read-only` enforcement |
| Safety | Context budgets | `--max-pages`, `--max-items`, `--max-bytes`, `--timeout` |

## Configuration

`atla` stores profile configuration in `~/.config/atla/config.toml` by default.
API tokens are stored through the OS keyring and are not written to config files.

Initial auth:

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --instance https://example.atlassian.net \
  --email you@example.com --token-stdin
atla auth status
atla config set default-project PROJ
atla config set alias.mine "jira search 'assignee = currentUser() order by updated desc'"
atla config list --output json
```

`atla` stores API tokens in the OS keyring by default. Atlassian tokens have a
configurable expiration of 1–365 days, so rotate them before expiry and re-run
`atla auth login` when needed. Unscoped tokens use the site URL. For a scoped token,
pass the tenant cloud ID; atla then routes Jira and Confluence through their respective
`api.atlassian.com/ex/{product}/{cloudId}` gateways while retaining the site URL for web links:

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --instance https://example.atlassian.net \
  --cloud-id 11111111-2222-3333-4444-555555555555 --email you@example.com --token-stdin
```

In headless or container environments where keyring access is unavailable, use an explicit
file-backed credential store or provide a token through the environment:

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --storage file \
  --instance https://example.atlassian.net --email you@example.com --token-stdin
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
atla jira issue comment add PROJ-123 "Please check the screenshot" --attachment ./screenshot.png
atla jira issue delete PROJ-123 --yes
atla jira issue fields --project PROJ --type Bug --required-only
atla jira board list --project PROJ
atla jira sprint active --board 84
```

Confluence examples:

```bash
atla confluence space list
atla confluence space create "Development" --key DEV --description "Team docs"
atla confluence page list --space DEV
atla confluence page view 67890 --format markdown
atla confluence page view 67890 --format markdown --preserve-table-options
atla confluence page create --space DEV --title "Meeting Notes" --body-file notes.html
atla confluence page create --space DEV --title "Inventory" --body-file inventory.md --representation markdown --numbered-table-rows
atla confluence page create --space DEV --title "Runbook" --body-file runbook.md --representation markdown --mention "Neo Hsu=abc-account-id"
atla confluence page update 67890 --title "Updated Notes"
atla confluence page move 67890 --parent 12345
atla confluence page label add 67890 runbook urgent
atla confluence page comment add 67890 "Looks good"
atla confluence page comment add 67890 "Please review the report" --representation markdown --attachment ./report.pdf
atla confluence blog create --space DEV --title "Release Notes" --body-file release.html
atla confluence search "type=page AND space=DEV"
atla confluence attachment upload 67890 ./diagram.png --comment "Updated diagram"
atla confluence attachment download 13579 --save-to ./diagram.png
```

Confluence v2 remains the primary generated client. Confluence search,
attachment upload, and page label mutation use scoped Confluence v1 REST
endpoints where v2 does not expose the required operation.

### Pagination

List and search commands treat `--limit N` as the maximum number of records to return for
that invocation. If more records exist, table output prints a ready-to-copy next command:

```text
More results available.
Next page:
  atla jira project list --limit 25 --page-token <TOKEN>
```

JSON output includes the same information under `pagination.nextPageToken` and
`pagination.nextCommand`. The token is opaque and validated against the command/query that
created it. Use `--all` instead of `--limit` when you intentionally want to fetch every
matching record. For agent runs, bound context and network work globally:

```bash
atla --read-only --max-pages 5 --max-items 200 --max-bytes 1000000 --timeout 30 \
  --output json jira search 'project = PROJ ORDER BY updated DESC'
```

When `--max-pages` or `--max-items` stops an `--all` request, atla emits a resume token instead
of pretending the result is complete. `--read-only` rejects local and remote mutations before
credentials or network access.

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

Override config paths for isolated test runs:

```bash
ATLA_CONFIG=/tmp/atla-config.toml cargo run -p atla -- config list
```

## Release

Releases are automated via cargo-dist. Pushing a version tag builds Linux,
macOS, and Windows binaries and publishes them to GitHub Releases.
