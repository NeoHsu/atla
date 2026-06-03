---
title: Output Formats and Global Flags
description: Global flags, output modes, environment variables, and automation patterns for atla.
---

# Output formats and global flags

These flags are available on every `atla` command:

| Flag | Type | Default | What it does |
| --- | --- | --- | --- |
| `-o, --output` | `json\|table\|csv\|keys` | `table` for tabular command output | Selects how results are printed |
| `--profile` | `STRING` | Active/default profile | Uses a specific auth/config profile |
| `--verbose` | boolean | `false` | Enables verbose HTTP/client output |
| `--dry-run` | boolean | `false` | Prints what would happen without making the change |
| `--no-input` | boolean | `false` | Disables interactive prompts for scripting/CI |

## `--output`

### `table`

Human-friendly tabular output. This is the best default for interactive use.

```bash
atla jira issue list --project PROJ
atla confluence page list --space ENG
```

### `json`

Pretty-printed JSON for machine-friendly workflows, debugging, and `jq`.

```bash
atla jira search 'project = PROJ' --output json
atla confluence page view 123456 --output json
```

### `csv`

Comma-separated rows for spreadsheets or shell pipelines.

```bash
atla jira issue list --project PROJ --output csv > jira-issues.csv
atla confluence blog list --space ENG --output csv > blog-posts.csv
```

### `keys`

Prints one identifier per line.
Use it for shell loops and quick selection.

- Jira usually prints issue keys such as `PROJ-123`
- Confluence usually prints content IDs, attachment IDs, or space keys depending on the command

```bash
atla jira issue list --project PROJ --output keys
atla confluence page list --space ENG --output keys
```

## `--limit`, `--all`, and pagination

List commands accept `--limit N` as a hard cap on returned items. `atla` paginates the
underlying API automatically and accumulates up to `N` records before printing — `--limit`
is **not** a single-page hint.

When the requested `--limit` is reached but the server still has more matches, a warning
is written to **stderr**:

```
warning: more issues match this query; increase --limit to fetch them (1000 returned)
```

Stdout is reserved for the records themselves, so `-o json`, `-o csv`, and `-o keys`
output is unaffected by the warning. Pipelines stay clean:

```bash
atla jira issue list --project PROJ --limit 5000 --output keys > keys.txt   # stdout
# stderr line, if any, appears in your terminal but not in keys.txt
```

To silence the warning in scripts that intentionally cap results, redirect stderr:

```bash
atla jira issue list --project PROJ --limit 100 --output json 2>/dev/null
```

### `--all`

When you want every matching record and would rather not pick a number, use `--all`
instead of `--limit`. It runs until the server reports no more results and **does not**
emit the truncation warning (you opted into the full fetch):

```bash
atla jira search 'project = PROJ AND statusCategory != Done' --all --output keys
atla confluence search 'type = page' --all --output json | jq '.results | length'
```

`--all` and `--limit` are mutually exclusive. A broad `--all` query can issue many HTTP
round trips (one per 100 items), so prefer narrower JQL/CQL filters when possible.

See [`jira.md`](./jira.md#pagination) and [`confluence.md`](./confluence.md#pagination) for
the full list of paginating commands.

## Piping and command-line tooling

### JSON + `jq`

```bash
atla jira search 'project = PROJ AND statusCategory != Done' --output json \
  | jq '.issues[] | {key, summary: .fields.summary}'

atla confluence search 'type = page AND space = ENG' --output json \
  | jq '.results[] | {id, title}'
```

### `keys` output in loops

```bash
for key in $(atla jira issue list --project PROJ --status 'To Do' --output keys); do
  echo "Need triage: $key"
done
```

### CSV exports

```bash
atla jira sprint issues 221 --fields summary,status,assignee --output csv > sprint-221.csv
```

## `--profile`

Profiles let you work against multiple Atlassian tenants or identities.

```bash
atla --profile work jira project list
atla --profile sandbox confluence space list
```

Profile resolution order:

1. `--profile <name>` if passed
2. `default.profile` from config
3. The first configured profile in the config file

## `--verbose`

Use `--verbose` when you need more request/response detail.
It is useful for troubleshooting auth, payload shape, and endpoint selection.

```bash
atla --verbose jira issue view PROJ-123 --output json
```

## `--dry-run`

`--dry-run` is global, but its most important guarantee is for mutating commands.

### Write commands

For create/update/delete/assign/transition/move/upload operations, `atla`:

- prints the REST call it would make
- does not send the write request
- lets you verify IDs, paths, and profiles first

```bash
atla --dry-run jira issue create --project PROJ --type Task --summary 'Prepare launch checklist'
atla --dry-run jira issue transition PROJ-123 --to Done
atla --dry-run confluence page move 123456 --parent 654321
atla --dry-run confluence attachment upload 123456 ./diagram.png
```

### Read commands

Many read commands also honor `--dry-run` by showing the request they would issue.
That is useful when debugging filters and generated URLs.

```bash
atla --dry-run jira search 'project = PROJ ORDER BY updated DESC'
atla --dry-run confluence search 'type = page AND space = ENG'
```

### Deletes and confirmations

For destructive commands, `--dry-run` lets you validate a delete without actually performing it.
Use it before the real command with `--yes`.

```bash
atla --dry-run jira issue delete PROJ-123 --yes
atla jira issue delete PROJ-123 --yes
```

## `--no-input`

`--no-input` disables prompts and interactive selectors.
Use it in CI, cron jobs, shell scripts, and any non-TTY environment.

It is especially important for:

- `atla auth login` when you want fully flag-driven auth
- `atla jira issue transition` when you do not want an interactive transition picker

```bash
atla --no-input jira issue transition PROJ-123 --to Done
atla --no-input auth login --instance https://example.atlassian.net --email you@example.com --token "$ATLA_TOKEN"
```

## CI and scripting patterns

### Safe automation defaults

```bash
atla --no-input --output json jira search 'project = PROJ AND statusCategory != Done'
atla --no-input --output json confluence search 'type = page AND label = runbook'
```

### Fail-safe change preview in CI

```bash
atla --no-input --dry-run jira sprint start 221 --start 2026-05-20 --end 2026-06-02
```

### Export to another tool

```bash
ISSUES_JSON=$(atla --no-input --output json jira issue list --project PROJ)
printf '%s\n' "$ISSUES_JSON" | jq '.issues | length'
```

### Batch actions from key-only output

```bash
atla jira issue list --project PROJ --status 'To Do' --output keys \
  | xargs -I{} atla --no-input --dry-run jira issue transition {} --to 'In Progress'
```

## Environment variables

| Variable | Purpose | Default |
| --- | --- | --- |
| `ATLA_TOKEN` | API token used instead of stored credentials | unset |
| `ATLA_API_TOKEN` | Backward-compatible alternative token variable | unset |
| `ATLA_CONFIG` | Path to the main config TOML file | `~/.config/atla/config.toml` |
| `ATLA_CREDENTIALS` | Path to the file credential store when using file-backed auth | `~/.config/atla/credentials.toml` |

### Token precedence

If `ATLA_TOKEN` or `ATLA_API_TOKEN` is set, `atla` uses that token before checking the configured keyring/file credential store.

```bash
ATLA_TOKEN="$ATLASSIAN_TOKEN" atla jira project list
```

### Isolated config for tests or automation

```bash
ATLA_CONFIG=$PWD/.atla-config.toml \
ATLA_CREDENTIALS=$PWD/.atla-credentials.toml \
atla --profile ci config list --output json
```

## Config and credential storage notes

- Main config lives in `config.toml`
- Tokens are stored in the OS keyring by default
- File-backed credentials are available for headless/container environments
- `atla auth login --storage file` switches the profile to file-backed credential storage

Example:

```bash
atla auth login --storage file \
  --instance https://example.atlassian.net \
  --email you@example.com \
  --token "$ATLA_TOKEN"
```
