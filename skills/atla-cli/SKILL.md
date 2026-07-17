---
name: atla-cli
description: >
  Reference and usage skill for the `atla` Atlassian CLI — a unified command-line tool
  for Jira and Confluence Cloud. Use this skill whenever the user wants to interact with
  Jira or Confluence from the terminal using `atla`: searching issues, creating or updating
  Jira tickets, managing sprints and boards, reading or publishing Confluence pages, managing
  spaces, uploading attachments, or any Atlassian Cloud automation. Also use when the user
  mentions `atla` commands, asks about JQL/CQL queries, or wants to script Atlassian
  workflows. Even if the user just says "create a Jira ticket", "check my sprint", "update
  the confluence page", "find issues assigned to me" — or in Chinese: 「開一張 Jira 票」、
  「建立工單」、「查 sprint」、「更新 Confluence 頁面」、「找我負責的 issue」、「搜尋
  Confluence」、「上傳附件到頁面」 — use this skill, because `atla` is the tool installed
  in this environment for those tasks.
---

# atla CLI Reference

`atla` is a Rust CLI for day-to-day Atlassian Cloud work. It covers Jira (projects, issues,
boards, sprints) and Confluence (spaces, pages, blogs, search, attachments) with profile-based
auth, multiple output formats, and a global `--dry-run` safety net.

## Prerequisites

Any command that exits with code `3` means auth/profile is not set up — its stderr
contains the exact `atla auth login ...` command to fix it; run that (or show it to the
user), verify with `atla auth status`, then retry the original command:

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --no-input \
  --instance https://example.atlassian.net --email you@example.com --token-stdin
atla auth status  # verify
```

For headless/CI environments, use `--storage file` or set `ATLA_TOKEN` env var.
Atlassian tokens expire after a configurable 1–365 days; rotate them before expiry. For a
scoped token, add `--cloud-id CLOUD_ID` to `auth login`; atla then selects separate Jira and
Confluence `api.atlassian.com/ex/{product}/{cloudId}` roots.

## Global Flags (available on every command)

| Flag | Purpose |
|------|---------|
| `-o, --output json\|table\|csv\|keys` | Output format (default: `table`) |
| `--profile NAME` | Use a specific auth profile |
| `--dry-run` | Preview the API call without executing |
| `--read-only` | Reject real local/remote mutations; allow dry-run previews |
| `--max-pages N` | Stop automatic pagination after N API pages |
| `--max-items N` | Return at most N records |
| `--max-bytes N` | Refuse oversized JSON output (requires `--output json`) |
| `--timeout SECONDS` | Bound each API request, upload, and download |
| `--no-input` | Disable interactive prompts (for scripts/CI) |
| `--verbose` | Show HTTP request/response details |

## Pagination, `--limit`, `--page-token`, and `--all`

On list/search commands, `--limit N` is a max-results cap for the current invocation.
`atla` paginates the underlying API automatically (Jira `nextPageToken`/`startAt`,
Confluence cursor or CQL `start`/`totalSize`) and accumulates up to `N` items before
returning — there is no need to write batch loops in the shell.

If `--limit` is reached but more matches exist server-side, `atla` exposes a next-page
token. Table output prints a ready-to-copy command, JSON output includes a `pagination`
object, and `keys`/`csv` keep stdout record-only while writing the next-page hint to
stderr:

```text
More results available.
Next page:
  atla jira search 'project = PROJ' --limit 50 --page-token <TOKEN>
```

Treat `--page-token` as opaque. Pass it back to the same command/query to continue; it is
validated against the command and query and fails fast if reused with different filters,
JQL/CQL, fields, or content IDs.

When you want every matching record without picking a number, use `--all`. Without a global
budget it runs until the server reports no more results and does not emit next-page metadata.
`--all` is mutually exclusive with both `--limit` and `--page-token`. If `--max-pages` or
`--max-items` stops it, atla emits a resume token. Prefer bounded runs for agents:

```bash
atla --read-only --max-pages 5 --max-items 200 --max-bytes 1000000 --timeout 30 \
  --output json jira search 'project = PROJ ORDER BY updated DESC'
```

## Command Tree Overview

### Core

- `atla auth login/logout/status/switch` — manage profiles and credentials
- `atla config set/get/list` — configuration and aliases
- `atla completion bash/elvish/fish/powershell/zsh` — shell completions

### Jira (`atla jira ...`)

- `project list/view/issue-types`
- `search <JQL>` — run JQL queries
- `issue list/create/view/update/edit/delete`
- `issue fields` — list create-meta fields (required flag, type, allowed values)
- `issue assign/transition`
- `issue comment add/list/update/delete` (supports `--attachment` and `--attachment-mode`)
- `issue attachment upload/list/download/delete`
- `issue link add/list/remove` (and `github-links` / `github-commits`)
- `issue worklog add/list`
- `board list/view`
- `sprint list/active/view/create/start/close/add/remove/issues`

### Confluence (`atla confluence ...`)

- `space list/view/create/update/delete`
- `page list/view/children/copy/create/update/move/delete`
- `page label list/add/remove`
- `page comment list/add/delete` (supports markdown options and page attachment references)
- `blog list/view/create/update/delete`
- `blog label list/add/remove`
- `blog comment list/add/delete`
- `search <CQL>` — run CQL queries
- `attachment list/view/upload/download/delete`

## Quick Patterns

### Find my open Jira work

```bash
atla jira search 'assignee = currentUser() AND statusCategory != Done ORDER BY updated DESC'
```

### Discover required fields before creating an issue

```bash
atla jira issue fields --project PROJ --type Bug --required-only
```

Use `issue fields` before `issue create` when a project has required custom fields.
The output shows each field's ID (for `--field`), type, and allowed values.

For **string** type fields, wrap the value as a JSON string:
`--field 'customfield_10166="5.1.0"'` — not `--field customfield_10166=5.1.0`

Plain values without quotes are auto-wrapped as `{"name":"..."}`, which the API rejects for string fields.

### Create a Jira issue

```bash
atla jira issue create --project PROJ --type Task --summary "Fix login bug" \
  --description "Users see 500 error on /login" --labels bug,urgent
```

### Transition an issue

```bash
atla jira issue transition PROJ-123 --to Done
```

### Comment with attachments

```bash
atla jira issue comment add PROJ-123 --body 'Please check logs' \
  --attachment ./error.log --attachment-mode link
atla confluence page comment add 123456 'Screenshot attached' \
  --attachment ./screenshot.png --attachment-mode auto
```

### Batch transition with keys output

```bash
atla jira issue list --project PROJ --status 'To Do' --output keys \
  | xargs -I{} atla --no-input jira issue transition {} --to 'In Progress'
```

### Read a Confluence page as Markdown

```bash
atla confluence page view 123456 --format markdown
```

### Create a Confluence page from a Markdown file

```bash
atla confluence page create --space ENG --title "Meeting Notes" \
  --body-file notes.md --representation markdown

# Add numbered rows support in Markdown tables
atla confluence page create --space ENG --title "Release Notes" \
  --body-file release-notes.md --representation markdown --numbered-table-rows
```

### Search Confluence

```bash
atla confluence search 'type = page AND space = ENG AND label = runbook' --limit 25
```

## Confluence: Always Use Attachments, Never External URLs

Confluence Cloud blocks externally referenced files (images, PDFs, etc.) via Content Security Policy. Any `<ri:url>` reference will silently fail to render without admin-level domain allowlisting — this applies to **all file types**, not just images.

**Rule: if a file needs to appear in a page, upload it first, then reference it by filename.**

```bash
# 1. Download the file locally
curl -sL -o /tmp/file.jpg "https://example.com/file.jpg"

# 2. Upload as a page attachment
atla confluence attachment upload PAGE_ID /tmp/file.jpg

# 3. Reference by attachment name in Storage Format — never by URL
# Images:  <ac:image><ri:attachment ri:filename="file.jpg"/></ac:image>
# Files:   <ac:structured-macro ac:name="view-file">
#             <ac:parameter ac:name="name"><ri:attachment ri:filename="file.pdf"/></ac:parameter>
#           </ac:structured-macro>
```

If a page was already created with external URL references, fix it:

```bash
atla confluence page update PAGE_ID --body-file fixed.xml --representation storage --version 4
```

## Common Traps

- `confluence page view <ID>` / `blog view <ID>` return **metadata only** (no body).
  To get content, pass `--format markdown` (or `storage` / `atlas-doc-format`).
- `page create/update` and comment `--body-file` default to `--representation storage`
  (XHTML). Feeding a Markdown file without `--representation markdown` produces broken
  content — always pass the flag when the source is Markdown.
- Runtime errors use classified exit codes: `2` usage, `3` auth, `4` not found,
  `5` safe-to-retry transient, `1` anything else. With `-o json`, stderr carries
  `{"schemaVersion": 1, "error": {"kind", "message", "status", "retryable"}}` instead of prose. For
  `kind=ambiguous_mutation`, query the target to verify whether it committed; never blindly
  repeat it. Auth errors include the exact `atla auth login ...` command to fix them.

## Safety Rules

- Use `--read-only` for discovery/planning agents. A dry-run mutation preview remains allowed.
  Persistent profile rules use `profiles.<name>.policy.mode/allow/deny`; deny wins over allow,
  and `*` is an anchored wildcard over the complete operation ID.
- Bound broad reads with `--max-pages`, `--max-items`, `--max-bytes`, and `--timeout`;
  bounded `--all` runs return a resume token.
- Always use `--dry-run` before destructive operations (delete, transition) when unsure.
  Jira issue and Confluence page/blog create/update support `--output json` operation plans with
  exact method, URL, and body; use them to verify field assembly and Markdown→ADF conversion.
  Supported plans can be saved with `atla plan ... --out FILE` and executed only via
  `atla apply FILE --yes`; apply verifies hash, expiry, active profile/site, policy, and URL.
- Always pass `--yes` for delete commands to skip confirmation, or omit it to get a prompt.
- Use `--no-input` in automated/scripted contexts to prevent hanging on prompts.
- For bulk operations, preview with `--dry-run` first, then remove it to execute.

## Detailed References

For full syntax, all flags, and advanced patterns for each command group, read:

- **Jira commands**: `references/jira.md` — projects, search, issues, comments, attachments, links, worklogs, boards, sprints, JQL reference, `--fields` usage, `--field KEY=VALUE` patterns
- **Confluence commands**: `references/confluence.md` — spaces, pages, blogs, search, attachments, labels, comments, CQL reference, content representations (storage/wiki/ADF/markdown)
- **Auth & config**: `references/auth-config.md` — login flows, multi-profile setup, aliases, environment variables, credential storage strategies
- **Saved plans**: `references/plans.md` — supported operations, plan/apply validation, hashes, and receipts

Read the appropriate reference file when you need exact flag syntax or edge-case details for a specific command.

## Configuration Essentials

Config file: `~/.config/atla/config.toml`

```bash
atla config set default-project PROJ          # default Jira project
atla config set default-space ENG             # default Confluence space
atla config set aliases.mine "jira search 'assignee = currentUser() order by updated desc'"
atla mine  # alias expands before parsing
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ATLA_TOKEN` / `ATLA_API_TOKEN` | API token override (highest priority) |
| `ATLA_CONFIG` | Custom config file path |
| `ATLA_CREDENTIALS` | Custom credentials file path |
| `ATLA_READ_ONLY` | Enforce mutation blocking |

## Output Formats for Scripting

| Format | Best for |
|--------|----------|
| `json` | `jq` pipelines, API payload inspection |
| `csv` | Spreadsheets, simple exports |
| `keys` | Shell loops, `xargs`, batch operations |
| `table` | Human reading (default) |

```bash
# JSON + jq
atla jira search 'project = PROJ' -o json | jq '.issues[] | {key, summary: .fields.summary}'

# Keys for loops
for key in $(atla jira issue list --project PROJ -o keys); do
  atla jira issue view "$key"
done
```
