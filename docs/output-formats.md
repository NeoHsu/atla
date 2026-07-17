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
| `--read-only` | boolean | `false` | Rejects real local or remote mutations; permits dry-run previews |
| `--max-pages` | positive integer | unlimited | Stops automatic pagination and preserves a resume token |
| `--max-items` | positive integer | command limit | Caps records returned by list/search operations |
| `--max-bytes` | positive integer | unlimited | With `--output json`, rejects oversized output before printing it |
| `--timeout` | positive integer seconds | API default | Bounds each request, upload, and download |
| `--no-input` | boolean | `false` | Disables interactive prompts for scripting/CI |

## `--output`

### `table`

Human-friendly tabular output. This is the best default for interactive use.

```bash
atla jira issue list --project PROJ
atla confluence page list --space ENG
```

### `json`

Pretty-printed JSON for machine-friendly workflows, debugging, and `jq`. Object outputs include
an additive integer `schemaVersion` (currently `1`).

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

When the requested `--limit` is reached but the server still has more matches, `atla`
prints or returns a next-page token. The next page uses the same `--limit` plus the
opaque `--page-token` value:

```text
More results available.
Next page:
  atla jira issue list --project PROJ --limit 100 --page-token <TOKEN>
```

For `table` output, the next-page command appears as a footer. For `json`, pagination
metadata is embedded in the object:

```json
{
  "schemaVersion": 1,
  "issues": [],
  "pagination": {
    "isLast": false,
    "nextPageToken": "...",
    "nextCommand": "atla jira search 'project = PROJ' --limit 50 --page-token ..."
  }
}
```

For `csv` and `keys`, stdout is reserved for records only; the next-page hint goes to
stderr so pipelines stay clean:

```bash
atla jira issue list --project PROJ --limit 100 --output keys > keys.txt
# next-page hint, if any, appears in your terminal but not in keys.txt
```

`--page-token` is validated against the command and query that produced it. Reusing a token
with different filters, JQL, CQL, fields, or content IDs fails fast instead of silently
returning the wrong page.

### `--all`

When you want every matching record and would rather not pick a number, use `--all`
instead of `--limit`. It runs until the server reports no more results and normally does not emit
a next-page token. If `--max-pages` or `--max-items` stops it first, the normal opaque resume token
is preserved:

```bash
atla jira search 'project = PROJ AND statusCategory != Done' --all --output keys
atla confluence search 'type = page' --all --output json | jq '.results | length'
```

`--all` is mutually exclusive with both `--limit` and `--page-token`. A broad `--all`
query can issue many HTTP round trips. If `--max-pages` or `--max-items` stops an `--all`
run, atla emits a resume token instead of reporting exhaustion.

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

- prints the REST call it would make;
- prints the final serializable payload for supported JSON-body mutations, including Jira issue
  and Confluence page/blog create/update operations;
- does not send the write request;
- lets you verify IDs, paths, profiles, and Markdown conversion first.

With `--output json`, Jira issue and Confluence page/blog create/update dry runs emit one
versioned operation plan containing the exact request method, URL, body, explicit preconditions,
and unresolved values:

```bash
atla --output json --dry-run jira issue create --project PROJ --type Task \
  --summary 'Prepare launch checklist'
```

Persist and apply supported plans explicitly:

```bash
atla plan jira issue create --project PROJ --type Task \
  --summary 'Prepare launch checklist' --out create-plan.json
atla apply create-plan.json --yes --output json
```

Apply verifies the plan hash, expiry (maximum 24 hours), input hashes, active profile/site, policy,
allowlisted operation and URL, then emits a mutation receipt. A plan digest is not a signature; do
not apply an untrusted plan.

Human-readable previews remain available without `--output json`. Unsupported JSON dry-runs fail
with exit code 2 and empty stdout, so JSON mode never falls back to prose:

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

## `--read-only` and context budgets

Use `--read-only` for discovery, audit, and planning agents. The operation registry rejects
config/auth writes and remote mutations before credentials are loaded or a request is sent.
`ATLA_READ_ONLY=1` enforces the same policy. Mutation dry-run previews remain available because
they do not execute the operation.

```bash
atla --read-only --output json jira issue view PROJ-123
```

Bound broad reads with `--max-pages`, `--max-items`, and `--max-bytes`. `--timeout` sets a
per-request deadline. An output that exceeds `--max-bytes` fails with exit code 2 before that
structured result is printed.

```bash
atla --read-only --max-pages 5 --max-items 200 --max-bytes 1000000 --timeout 30 \
  --output json confluence search 'type = page AND label = runbook'
```

## `--no-input`

`--no-input` disables prompts and interactive selectors.
Use it in CI, cron jobs, shell scripts, and any non-TTY environment.

It is especially important for:

- `atla auth login` when you want fully flag-driven auth
- `atla jira issue transition` when you do not want an interactive transition picker

```bash
atla --no-input jira issue transition PROJ-123 --to Done
printf '%s\n' "$ATLA_TOKEN" | atla --no-input auth login \
  --instance https://example.atlassian.net --email you@example.com --token-stdin
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
| `ATLA_READ_ONLY` | Enforce mutation blocking | unset/false |

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
printf '%s\n' "$ATLA_TOKEN" | atla auth login --no-input --storage file \
  --instance https://example.atlassian.net \
  --email you@example.com \
  --token-stdin
```
