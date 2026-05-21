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
  the confluence page", or "find issues assigned to me" — use this skill, because `atla`
  is the tool installed in this environment for those tasks.
---

# atla CLI Reference

`atla` is a Rust CLI for day-to-day Atlassian Cloud work. It covers Jira (projects, issues,
boards, sprints) and Confluence (spaces, pages, blogs, search, attachments) with profile-based
auth, multiple output formats, and a global `--dry-run` safety net.

## Prerequisites

The user must have `atla` installed and authenticated. If auth fails, guide them through:

```bash
atla auth login --instance https://SITE.atlassian.net --email USER@example.com --token "$ATLASSIAN_TOKEN"
atla auth status  # verify
```

For headless/CI environments, use `--storage file` or set `ATLA_TOKEN` env var.

## Global Flags (available on every command)

| Flag | Purpose |
|------|---------|
| `-o, --output json\|table\|csv\|keys` | Output format (default: `table`) |
| `--profile NAME` | Use a specific auth profile |
| `--dry-run` | Preview the API call without executing |
| `--no-input` | Disable interactive prompts (for scripts/CI) |
| `--verbose` | Show HTTP request/response details |

## Command Tree Overview

### Core
- `atla auth login/logout/status/switch` — manage profiles and credentials
- `atla config set/get/list` — configuration and aliases
- `atla completion bash/zsh/fish/powershell` — shell completions

### Jira (`atla jira ...`)
- `project list/view/issue-types`
- `search <JQL>` — run JQL queries
- `issue list/create/view/update/edit/delete`
- `issue assign/transition`
- `issue comment add/list/update/delete`
- `issue attachment upload/list/download/delete`
- `issue link add/list/remove`
- `issue worklog add/list`
- `board list/view`
- `sprint list/active/view/create/start/close/add/remove/issues`

### Confluence (`atla confluence ...`)
- `space list/view/create/update/delete`
- `page list/view/children/copy/create/update/move/delete`
- `page label list/add/remove`
- `page comment list/add/delete`
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

### Create a Jira issue
```bash
atla jira issue create --project PROJ --type Task --summary "Fix login bug" \
  --description "Users see 500 error on /login" --labels bug,urgent
```

### Transition an issue
```bash
atla jira issue transition PROJ-123 --to Done
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
atla confluence page update PAGE_ID --body-file fixed.xml --representation storage --version N
```

## Safety Rules

- Always use `--dry-run` before destructive operations (delete, transition) when unsure.
- Always pass `--yes` for delete commands to skip confirmation, or omit it to get a prompt.
- Use `--no-input` in automated/scripted contexts to prevent hanging on prompts.
- For bulk operations, preview with `--dry-run` first, then remove it to execute.

## Detailed References

For full syntax, all flags, and advanced patterns for each command group, read:

- **Jira commands**: `references/jira.md` — projects, search, issues, comments, attachments, links, worklogs, boards, sprints, JQL reference, `--fields` usage, `--field KEY=VALUE` patterns
- **Confluence commands**: `references/confluence.md` — spaces, pages, blogs, search, attachments, labels, comments, CQL reference, content representations (storage/wiki/ADF/markdown)
- **Auth & config**: `references/auth-config.md` — login flows, multi-profile setup, aliases, environment variables, credential storage strategies

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
