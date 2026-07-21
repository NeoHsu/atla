---
title: Agent Reference
description: Compact complete reference for AI agents and automation using atla.
---

# Agent reference

## 1. Overview

`atla` is a unified Atlassian Cloud CLI for Jira and Confluence, written in Rust. It provides profile-based authentication, machine-friendly and human-friendly output modes, global dry-run safety, and command coverage for Jira projects/issues/boards/sprints plus Confluence spaces/pages/blogs/search/comments/labels/attachments.

Token expiry is configured by Atlassian outside atla. Record the expiration shown when creating a
token and rotate it before that date; atla reports availability/source, not expiry. Unscoped
tokens use the site URL. For a scoped token, pass `--cloud-id` during login; the same
profile then routes Jira and Confluence through their product-specific
`api.atlassian.com/ex/{product}/{cloudId}` gateway roots.

### Install the bundled skill

The repo contains an installable AI-agent skill at `skills/atla-cli`.

Install the skill tag that exactly matches `atla 0.6.0`:

```bash
npx skills add https://github.com/NeoHsu/atla/tree/v0.6.0 --skill atla-cli
```

Released CLI/skill versions are exact lockstep. Never install a released skill from an unversioned
default branch; after a CLI upgrade, reinstall from the matching tag. Every skill execution begins
with `atla doctor --skill-version 0.6.0 --output json` and stops before config, credentials, or
network access when versions differ.

From a local checkout of this repo:

```bash
npx skills add . --skill atla-cli
```

For non-interactive setup across all supported agents:

```bash
npx skills add . --skill atla-cli --agent '*' -y
```

Use `--copy` if you want the installed skill to be a standalone copy instead of a symlink
back to the repo checkout.

## 2. Command Tree

### Core

- `atla auth login` — create/update a profile; use `--token-stdin` in automation and add `--cloud-id <ID>` for a scoped token.
- `atla auth discover --site <URL>` — discover cloud ID and product gateway roots.
- `atla auth logout --yes` — remove stored credentials for the active profile.
- `atla auth status` — show profile configuration and token availability/source.
- `atla auth switch <profile>` — make a profile the default.
- `atla config set <key> <value>` — set a config key.
- `atla config get <key>` — read a config key.
- `atla config list` — print all config entries.
- `atla doctor` — inspect config, profile, credential source, API target, and policy locally; add
  `--skill-version <VERSION>` for an exact fail-closed CLI/skill check or `--network` to test
  tenant reachability/cloud-ID discovery.
- `atla explain-policy <operation-id>` — explain deny → allow → mode evaluation plus global
  `--read-only` for one stable operation ID.
- `atla operation list` — list stable IDs, methods, risk, pagination, dry-run, and retry metadata.
- `atla schema list` / `atla schema print <name>` — discover or print bundled JSON contracts.
- `atla plan jira|confluence ... --out <FILE>` — write an expiring, hashed mutation plan.
- `atla apply <FILE> --yes` — validate and execute an allowlisted saved plan.
- `atla completion <shell>` — generate shell completions.

### Jira

- `atla jira project list` — list Jira projects.
- `atla jira project view <KEY>` — show one Jira project.
- `atla jira project issue-types <KEY>` — list valid issue types for a project.
- `atla jira search <JQL>` — run JQL search.
- `atla jira issue list` — list issues with filters or custom JQL.
- `atla jira issue create` — create an issue.
- `atla jira issue update <KEY>` — update summary, description, fields, or labels.
- `atla jira issue edit <KEY>` — alias for `issue update`.
- `atla jira issue view <KEY>` — show an issue or open it in the browser.
- `atla jira issue delete <KEY>` — delete an issue.
- `atla jira issue assign <KEY>` — assign, reassign, or unassign an issue.
- `atla jira issue transition <KEY>` — list/select/apply workflow transitions.
- `atla jira issue fields` — list create-meta fields for a project/type.
- `atla jira issue comment add <KEY>` — add a comment.
- `atla jira issue comment list <KEY>` — list comments.
- `atla jira issue comment update <KEY> <COMMENT_ID>` — edit a comment.
- `atla jira issue comment delete <KEY> <COMMENT_ID>` — delete a comment.
- `atla jira issue attachment upload <KEY>` — upload a file.
- `atla jira issue attachment list <KEY>` — list attachments.
- `atla jira issue attachment download <KEY_OR_ID>` — download one or all attachments.
- `atla jira issue attachment delete <ATTACHMENT_ID>` — delete an attachment.
- `atla jira issue link add <KEY>` — create an issue link.
- `atla jira issue link list <KEY>` — list linked issues.
- `atla jira issue link remove <LINK_ID>` — remove a link.
- `atla jira issue link github-links <KEY>` — list GitHub PRs from the development panel.
- `atla jira issue link github-commits <KEY>` — list GitHub commits from the development panel.
- `atla jira issue worklog add <KEY>` — add logged time.
- `atla jira issue worklog list <KEY>` — list worklogs.
- `atla jira board list` — list Jira Software boards.
- `atla jira board view <ID>` — show a board.
- `atla jira sprint list --board <ID>` — list board sprints.
- `atla jira sprint active --board <ID>` — list active sprints.
- `atla jira sprint view <ID>` — show a sprint.
- `atla jira sprint create` — create a sprint.
- `atla jira sprint start <ID>` — start a sprint.
- `atla jira sprint close <ID>` — close a sprint.
- `atla jira sprint add <ID>` — add issues to a sprint.
- `atla jira sprint remove <ID>` — move issues back to backlog.
- `atla jira sprint issues <ID>` — list issues in a sprint.

### Confluence

- `atla confluence space list` — list spaces.
- `atla confluence space view <KEY>` — show one space.
- `atla confluence space create <NAME>` — create a space.
- `atla confluence space update <KEY>` — update name/description.
- `atla confluence space delete <KEY>` — delete a space.
- `atla confluence page list` — list pages.
- `atla confluence page view <ID>` — show a page body or metadata.
- `atla confluence page children <ID>` — list child pages.
- `atla confluence page copy <SOURCE_ID>` — clone a page into another location.
- `atla confluence page create` — create a page.
- `atla confluence page update <ID>` — update title/body/version/parent.
- `atla confluence page delete <ID>` — delete a page.
- `atla confluence page move <ID>` — move a page to a new parent.
- `atla confluence page label list <PAGE_ID>` — list labels.
- `atla confluence page label add <PAGE_ID>` — add labels.
- `atla confluence page label remove <PAGE_ID> <LABEL> --yes` — remove a label.
- `atla confluence page comment list <PAGE_ID>` — list comments.
- `atla confluence page comment add <PAGE_ID>` — add a comment.
- `atla confluence page comment delete <PAGE_ID> <COMMENT_ID>` — delete a comment.
- `atla confluence blog list` — list blog posts.
- `atla confluence blog view <ID>` — show a blog post.
- `atla confluence blog create` — create a blog post.
- `atla confluence blog update <ID>` — update a blog post.
- `atla confluence blog delete <ID>` — delete a blog post.
- `atla confluence blog label list <BLOG_ID>` — list blog labels.
- `atla confluence blog label add <BLOG_ID>` — add blog labels.
- `atla confluence blog label remove <BLOG_ID> <LABEL> --yes` — remove a blog label.
- `atla confluence blog comment list <BLOG_ID>` — list blog comments.
- `atla confluence blog comment add <BLOG_ID>` — add a blog comment.
- `atla confluence blog comment delete <BLOG_ID> <COMMENT_ID>` — delete a blog comment.
- `atla confluence search <CQL>` — run Confluence Query Language search.
- `atla confluence attachment list <PAGE_ID>` — list page attachments.
- `atla confluence attachment view <ATTACHMENT_ID>` — inspect an attachment.
- `atla confluence attachment upload <PAGE_ID> <FILE>` — upload a file to a page.
- `atla confluence attachment download <ATTACHMENT_ID>` — download a file.
- `atla confluence attachment delete <ATTACHMENT_ID>` — delete an attachment.

## 3. Global Flags

| Flag | Type | Default | Notes |
| --- | --- | --- | --- |
| `-o, --output` | `json\|table\|csv\|keys` | `table` for tabular results | Human or machine output mode |
| `--profile` | string | active/default profile | Selects auth/config profile |
| `--verbose` | boolean | `false` | Enables verbose client logging |
| `--dry-run` | boolean | `false` | Prints the request and serializable final payload, then skips the mutation |
| `--read-only` | boolean | `false` | Rejects real local/remote mutations before credential/network access; permits dry-run previews; also settable with `ATLA_READ_ONLY` |
| `--max-pages` | positive integer | unlimited | Stops automatic pagination after this many API pages and preserves a resume token |
| `--max-items` | positive integer | command limit | Caps records returned by a list operation |
| `--max-bytes` | positive integer | unlimited | Requires JSON output; fails before printing an oversized document |
| `--timeout` | positive integer seconds | API default | Per-request deadline, including uploads and downloads |
| `--no-input` | boolean | `false` | Disables prompts and interactive selection |

### Exit codes and error output

| Code | Kind | Meaning |
| --- | --- | --- |
| `0` | — | Success |
| `1` | `other`, `ambiguous_mutation` | Non-retryable business/IO failures, or a mutation whose remote outcome must be verified |
| `2` | `usage`, `version_mismatch` | Invalid arguments/policy, or an exact CLI/skill mismatch |
| `3` | `auth` | Missing/invalid credentials or profile (HTTP 401/403) |
| `4` | `not_found` | Resource does not exist (HTTP 404) |
| `5` | `retryable` | Safe-to-retry transient failure, including HTTP 429 and read/idempotent request failures |

With `-o json`, object outputs carry additive `schemaVersion: 1`, and runtime errors are emitted
to stderr as a machine-readable object:

```json
{
  "schemaVersion": 1,
  "error": {
    "kind": "not_found",
    "message": "failed to load Jira issue `NOPE-1`: Atlassian API returned 404 Not Found: Issue does not exist or you do not have permission to see it.",
    "status": 404,
    "retryable": false
  }
}
```

Agents should retry (with backoff) only when the exit code is `5` or
`error.retryable` is `true`. For `ambiguous_mutation`, query the target resource to determine
whether the mutation committed; do not blindly repeat it.

For machine-readable planning, use `--output json --dry-run`. Jira issue and Confluence page/blog
create/update commands emit `planVersion: 1`, operation/profile identity, exact request method/URL/body,
preconditions, unresolved values, and `mutating: true` without network access. Supported operations
can be persisted with `atla plan ... --out FILE`; `atla apply FILE --yes` validates hash, expiry,
input files, profile/site, policy, and allowlisted request shape before execution. Successful mutation
JSON objects include receipt fields: `operation`, `profile`, `target`, `requestId`, and `completedAt`.

### Pagination

Every `--limit N` flag is a "max-results" cap. `atla` paginates the underlying API
internally (Jira `startAt`/`nextPageToken`, Confluence v2 cursor / v1 CQL `start`) until
`N` items are collected or the server signals exhaustion. Agents can pass `--limit 5000`
without writing their own batch loop.

If the limit is reached before the server runs out, `atla` exposes a next-page token.
Table output prints a ready-to-copy command, JSON output includes a `pagination` object,
and `keys`/`csv` keep stdout record-only while writing the next-page hint to stderr:

```text
More results available.
Next page:
  atla jira search 'project = PROJ' --limit 50 --page-token <TOKEN>
```

Agents should treat `--page-token` as opaque. Pass it back to the same command/query to
continue; tokens are validated against the command and query and fail fast if reused with
different filters, JQL/CQL, fields, or content IDs.

#### `--all`

For unbounded "fetch everything" runs, use `--all` instead of guessing `--limit`. Without a
global context budget it follows the cursor until the server is empty and emits no next-page
metadata. `--all` is mutually exclusive with both `--limit` and `--page-token`. If
`--max-pages` or `--max-items` stops the run, atla emits a resume token instead of reporting
exhaustion. Prefer bounded agent calls:

```bash
atla --read-only --max-pages 5 --max-items 200 --max-bytes 1000000 --timeout 30 \
  --output json jira search 'project = PROJ ORDER BY updated DESC'
```

```bash
atla jira issue list --jql "project=PROJ" --all --output keys
atla confluence search 'type = page' --all --output json
```

Affected commands (`--limit`, `--all`, and `--page-token` supported on each):
`jira project list`, `jira search`, `jira issue list`, `jira issue comment list`,
`jira issue worklog list`, `jira board list`, `jira sprint list`, `jira sprint active`,
`jira sprint issues`, `confluence space list`, `confluence page list`,
`confluence page children`, `confluence blog list`, `confluence page comment list`,
`confluence blog comment list`, `confluence page label list`,
`confluence blog label list`, `confluence attachment list`, `confluence search`.

### Local discovery and diagnostics

These commands are read-only and bypass profile allow/deny policy so users can diagnose a locked
configuration. Only `doctor --network` sends a request; it calls the unauthenticated tenant-info
endpoint and never prints a token.

| Command | Args | Flags | Description | Example |
| --- | --- | --- | --- | --- |
| `doctor` | none | `--network`, `--skill-version` | Report local configuration and optionally verify exact CLI/skill compatibility or tenant discovery. | `atla doctor --skill-version 0.6.0 --output json` |
| `explain-policy` | `<OPERATION_ID>` | none | Show the matching deny/allow rule or mode and global read-only result. | `atla --profile agent explain-policy jira.issue.create --output json` |
| `operation list` | none | none | List the complete stable operation registry and safety metadata. | `atla operation list --output json` |
| `schema list` | none | none | List every bundled public JSON schema. | `atla schema list --output json` |
| `schema print` | `<NAME>` | JSON/default output only | Print the exact bundled schema without adding result-envelope fields. | `atla schema print error-v1 --output json` |

`doctor` reports check-level `ok`, `warning`, `error`, or `skipped` status and an aggregate
`healthy` boolean. Ordinary unhealthy checks remain diagnostic-only. An exact `--skill-version`
mismatch is the exception: it exits `2` with `kind=version_mismatch`, emits structured
`skillCompatibility` remediation, and performs no config, credential, or network checks.
`schema print` accepts both `error-v1` and `error-v1.schema.json`; table/csv/keys output is rejected.

## 4. Jira Commands

| Command | Args | Flags | Description | Example |
| --- | --- | --- | --- | --- |
| `jira project list` | none | `--query`, `--limit`, `--page-token`, `--all` | List projects, optionally filtered by name/key text. | `atla jira project list --query platform --limit 25` |
| `jira project view` | `<KEY>` | none | Show project metadata. | `atla jira project view PROJ` |
| `jira project issue-types` | `<KEY>` | none | List issue types valid for project create flows. | `atla jira project issue-types PROJ` |
| `jira search` | `<JQL>` | `--limit`, `--page-token`, `--fields`, `--all` | Run JQL search directly. | `atla jira search 'project = PROJ ORDER BY updated DESC' --fields summary,status` |
| `jira issue list` | none | `--project`, `--status`, `--type`, `--assignee`, `--jql`, `--limit`, `--page-token`, `--fields`, `--all` | List issues by filters or custom JQL. | `atla jira issue list --project PROJ --status 'In Progress'` |
| `jira issue create` | none | `--project`, `--type`, `--summary`, `--description`, `--description-file`, `--field`, `--labels` | Create an issue. | `atla jira issue create --project PROJ --type Task --summary 'Fix login'` |
| `jira issue update` | `<KEY>` | `--summary`, `--description`, `--description-file`, `--field`, `--labels` | Update an issue. Alias: `edit`. | `atla jira issue update PROJ-123 --labels add:urgent` |
| `jira issue fields` | none | `--project`, `--type`, `--required-only` | List create-meta fields (ID, type, allowed values) for `issue create`. | `atla jira issue fields --project PROJ --type Bug --required-only` |
| `jira issue view` | `<KEY>` | `--web`, `--fields`, `--with-github` | Show issue details or open in browser. | `atla jira issue view PROJ-123 --fields summary,status` |
| `jira issue delete` | `<KEY>` | `--delete-subtasks`, `--yes` | Delete an issue. | `atla jira issue delete PROJ-123 --yes` |
| `jira issue assign` | `<KEY>` | `--to`, `--account-id`, `--unassign` | Assign or clear assignee. | `atla jira issue assign PROJ-123 --to me` |
| `jira issue transition` | `<KEY>` | `--to`, `--field` | Apply workflow transition; can prompt unless `--no-input`. | `atla jira issue transition PROJ-123 --to Done` |
| `jira issue comment add` | `<KEY>` | `BODY`, `--body`, `--body-file`, `--attachment`, `--attachment-mode` | Add a comment; exactly one body source is required. | `atla jira issue comment add PROJ-123 --body 'Ready for review'` |
| `jira issue comment list` | `<KEY>` | `--limit`, `--page-token`, `--all` | List comments. | `atla jira issue comment list PROJ-123 --limit 10` |
| `jira issue comment update` | `<KEY> <COMMENT_ID>` | `--body`, `--body-file` | Update a comment; exactly one body source is required. | `atla jira issue comment update PROJ-123 10001 --body 'Merged'` |
| `jira issue comment delete` | `<KEY> <COMMENT_ID>` | `--yes` | Delete a comment. | `atla jira issue comment delete PROJ-123 10001 --yes` |
| `jira issue attachment upload` | `<KEY>` | `--file` | Upload attachment. | `atla jira issue attachment upload PROJ-123 --file ./bug.png` |
| `jira issue attachment list` | `<KEY>` | none | List attachments. | `atla jira issue attachment list PROJ-123` |
| `jira issue attachment download` | `<KEY_OR_ID>` | `--all`, `--dest` | Download one attachment or all issue attachments. | `atla jira issue attachment download PROJ-123 --all --dest ./downloads` |
| `jira issue attachment delete` | `<ATTACHMENT_ID>` | `--yes` | Delete attachment. | `atla jira issue attachment delete 10020 --yes` |
| `jira issue link add` | `<KEY>` | `--type`, `--target` | Create issue link. | `atla jira issue link add PROJ-123 --type Blocks --target PROJ-456` |
| `jira issue link list` | `<KEY>` | none | List issue links. | `atla jira issue link list PROJ-123` |
| `jira issue link remove` | `<LINK_ID>` | `--yes` | Remove issue link. | `atla jira issue link remove 10500 --yes` |
| `jira issue link github-links` | `<KEY>` | none | List GitHub PRs from the Jira development panel. | `atla jira issue link github-links PROJ-123` |
| `jira issue link github-commits` | `<KEY>` | none | List GitHub commits from the Jira development panel. | `atla jira issue link github-commits PROJ-123` |
| `jira issue worklog add` | `<KEY>` | `--time`, `--comment`, `--started` | Add time spent entry. | `atla jira issue worklog add PROJ-123 --time 45m --comment 'Debugged callback'` |
| `jira issue worklog list` | `<KEY>` | `--limit`, `--page-token`, `--all` | List worklogs. | `atla jira issue worklog list PROJ-123 --limit 10` |
| `jira board list` | none | `--project`, `--type`, `--name`, `--limit`, `--page-token`, `--all` | List Jira Software boards. | `atla jira board list --project PROJ --type scrum` |
| `jira board view` | `<ID>` | none | Show one board. | `atla jira board view 84` |
| `jira sprint list` | none | `--board`, `--state`, `--limit`, `--page-token`, `--all` | List sprints for a board. | `atla jira sprint list --board 84 --state active` |
| `jira sprint active` | none | `--board`, `--limit`, `--page-token`, `--all` | Show active sprints for a board. | `atla jira sprint active --board 84` |
| `jira sprint view` | `<ID>` | none | Show one sprint. | `atla jira sprint view 221` |
| `jira sprint create` | none | `--board`, `--name`, `--start`, `--end`, `--goal` | Create a sprint. | `atla jira sprint create --board 84 --name 'Sprint 42'` |
| `jira sprint start` | `<ID>` | `--start`, `--end` | Start a sprint. | `atla jira sprint start 221 --start 2026-05-20 --end 2026-06-02` |
| `jira sprint close` | `<ID>` | none | Close a sprint. | `atla jira sprint close 221` |
| `jira sprint add` | `<ID>` | `--issues` / `--issue` | Add issues to sprint. | `atla jira sprint add 221 --issues PROJ-123,PROJ-124` |
| `jira sprint remove` | `<ID>` | `--issues` / `--issue` | Remove issues from sprint back to backlog. | `atla jira sprint remove 221 --issues PROJ-124` |
| `jira sprint issues` | `<ID>` | `--limit`, `--page-token`, `--fields`, `--all` | List issues in a sprint. | `atla jira sprint issues 221 --fields summary,status,assignee` |

## 5. Confluence Commands

| Command | Args | Flags | Description | Example |
| --- | --- | --- | --- | --- |
| `confluence space list` | none | `--key`, `--limit`, `--page-token`, `--all` | List spaces. | `atla confluence space list --key ENG --limit 10` |
| `confluence space view` | `<KEY>` | none | Show one space; JSON includes optional `spaceOwnerId`. | `atla confluence space view ENG --output json` |
| `confluence space create` | `<NAME>` | `--key`, `--alias`, `--description`, `--description-file`, `--private` | Create a space; exactly one of `--key`/`--alias` is required. | `atla confluence space create 'Engineering Docs' --key ENG` |
| `confluence space update` | `<KEY>` | `--name`, `--description`, `--description-file` | Update space metadata; at least one update field is required. | `atla confluence space update ENG --name 'Engineering Knowledge Base'` |
| `confluence space delete` | `<KEY>` | `--yes` | Delete a space. | `atla confluence space delete ENG --yes` |
| `confluence page list` | none | `-s/--space`, `--space-id`, `--title`, `--limit`, `--page-token`, `--all` | List pages. | `atla confluence page list --space ENG --title Runbook` |
| `confluence page view` | `<ID>` | `--web`, `--format`, `--metadata-only`, `--fields`, `--max-chars`, `--preserve-table-options`, `--with-attachments` | Metadata is the bounded default and includes `bodyCommand`; `--format` fetches the body. JSON projection uses `--fields`; `--max-chars` bounds body text. | `atla confluence page view 123456 --format markdown --max-chars 50000` |
| `confluence page children` | `<ID>` | `--depth`, `--limit`, `--page-token`, `--all` | List page children or descendants. | `atla confluence page children 123456 --depth 2` |
| `confluence page copy` | `<SOURCE_ID>` | `--title`, `-s/--space`, `--space-id`, `--parent`, `--root-level` | Copy a page. | `atla confluence page copy 123456 --title 'Template Copy' --space ENG` |
| `confluence page create` | none | `-s/--space`, `--space-id`, `--title`, `--parent`, `--root-level`, `--body`, `--body-file`, `--representation`, `--numbered-table-rows`, `--mention`, `--resolve-mentions`, `--draft`, `--private` | Create a page. | `atla confluence page create --space ENG --title 'Checklist' --body-file docs/checklist.md --representation markdown` |
| `confluence page update` | `<ID>` | `--title`, `--parent`, `--body`, `--body-file`, `--representation`, `--numbered-table-rows`, `--mention`, `--resolve-mentions`, `--version`, `--message`, `--draft` | Update page title/body/version. | `atla confluence page update 123456 --title 'Checklist v2'` |
| `confluence page delete` | `<ID>` | `--purge`, `--draft`, `--yes` | Delete page. | `atla confluence page delete 123456 --yes` |
| `confluence page move` | `<ID>` | `--parent` | Move page under a new parent. | `atla confluence page move 123456 --parent 654321` |
| `confluence page label list` | `<PAGE_ID>` | `--prefix`, `--limit`, `--page-token`, `--all` | List page labels. | `atla confluence page label list 123456 --limit 20` |
| `confluence page label add` | `<PAGE_ID> LABEL...` | none | Add page labels. | `atla confluence page label add 123456 runbook urgent` |
| `confluence page label remove` | `<PAGE_ID> <LABEL>` | `--yes` | Remove page label. | `atla confluence page label remove 123456 urgent --yes` |
| `confluence page comment list` | `<PAGE_ID>` | `--limit`, `--page-token`, `--all` | List page comments. | `atla confluence page comment list 123456 --limit 10` |
| `confluence page comment add` | `<PAGE_ID>` | `BODY`, `--body`, `--body-file`, `--parent`, `--representation`, `--numbered-table-rows`, `--mention`, `--resolve-mentions`, `--attachment`, `--attachment-mode` | Add page comment; exactly one body source is required. | `atla confluence page comment add 123456 'Looks good'` |
| `confluence page comment delete` | `<PAGE_ID> <COMMENT_ID>` | `--yes` | Delete page comment. | `atla confluence page comment delete 123456 78910 --yes` |
| `confluence blog list` | none | `-s/--space`, `--space-id`, `--title`, `--limit`, `--page-token`, `--all` | List blog posts. | `atla confluence blog list --space ENG --limit 10` |
| `confluence blog view` | `<ID>` | `--format`, `--metadata-only`, `--fields`, `--max-chars` | Metadata is the bounded default; body views support JSON projection and truncation. | `atla confluence blog view 234567 --format markdown --max-chars 50000` |
| `confluence blog create` | none | `-s/--space`, `--space-id`, `--title`, `--body`, `--body-file`, `--representation`, `--draft`, `--private` | Create a blog post. | `atla confluence blog create --space ENG --title 'Release Notes' --body-file docs/release.md --representation markdown` |
| `confluence blog update` | `<ID>` | `--title`, `--body`, `--body-file`, `--representation`, `--version`, `--message`, `--draft` | Update a blog post. | `atla confluence blog update 234567 --message 'Add known issues'` |
| `confluence blog delete` | `<ID>` | `--purge`, `--draft`, `--yes` | Delete a blog post. | `atla confluence blog delete 234567 --yes` |
| `confluence blog label list` | `<BLOG_ID>` | `--prefix`, `--limit`, `--page-token`, `--all` | List blog labels. | `atla confluence blog label list 234567 --limit 20` |
| `confluence blog label add` | `<BLOG_ID> LABEL...` | none | Add blog labels. | `atla confluence blog label add 234567 release-notes engineering` |
| `confluence blog label remove` | `<BLOG_ID> <LABEL>` | `--yes` | Remove blog label. | `atla confluence blog label remove 234567 engineering --yes` |
| `confluence blog comment list` | `<BLOG_ID>` | `--limit`, `--page-token`, `--all` | List blog comments. | `atla confluence blog comment list 234567 --limit 10` |
| `confluence blog comment add` | `<BLOG_ID>` | `BODY`, `--body`, `--body-file`, `--parent`, `--representation` | Add blog comment; exactly one body source is required. | `atla confluence blog comment add 234567 'Ship after QA sign-off'` |
| `confluence blog comment delete` | `<BLOG_ID> <COMMENT_ID>` | `--yes` | Delete blog comment. | `atla confluence blog comment delete 234567 78910 --yes` |
| `confluence search` | `<CQL>` | `--limit`, `--page-token`, `--all` | Run CQL search. | `atla confluence search 'type = page AND space = ENG' --limit 25` |
| `confluence attachment list` | `<PAGE_ID>` | `--filename`, `--limit`, `--page-token`, `--all` | List page attachments. | `atla confluence attachment list 123456 --filename diagram` |
| `confluence attachment view` | `<ATTACHMENT_ID>` | none | Show attachment metadata. | `atla confluence attachment view 987654` |
| `confluence attachment upload` | `<PAGE_ID> <FILE>` | `--comment`, `--minor-edit` | Upload attachment to page. | `atla confluence attachment upload 123456 ./diagram.png --minor-edit` |
| `confluence attachment download` | `<ATTACHMENT_ID>` | `--save-to`/`-f` | Download attachment. | `atla confluence attachment download 987654 --save-to ./downloads/diagram.png` |
| `confluence attachment delete` | `<ATTACHMENT_ID>` | `--purge`, `--yes` | Delete attachment. | `atla confluence attachment delete 987654 --yes` |

## 6. Output Formats

| Format | Use when | Notes |
| --- | --- | --- |
| `table` | Human CLI sessions | Default for record-style output |
| `json` | `jq`, scripts, API payload inspection | Pretty-printed JSON |
| `csv` | Spreadsheets or simple exports | Header row included |
| `keys` | Shell loops, `xargs`, batch automation | Prints one key/ID per line |

## 7. Configuration Keys

| Key | Scope | Meaning | Example |
| --- | --- | --- | --- |
| `schema-version` | global, read-only | Current config schema (`2`) | `atla config get schema-version` |
| `default.profile` | global | Default active profile name | `atla config set default.profile work` |
| `default-profile` | global alias | CLI-friendly alias for `default.profile` | `atla config set default-profile work` |
| `instance` | active profile shorthand | Base Atlassian site URL for active profile | `atla config set instance https://example.atlassian.net` |
| `email` | active profile shorthand | Atlassian account email for active profile | `atla config set email you@example.com` |
| `credential-store` / `credential_store` | active profile shorthand | Token storage backend: `keyring` or `file` | `atla config set credential-store file` |
| `cloud-id` / `cloud_id` | active profile shorthand | Scoped-token tenant ID; empty clears gateway routing | `atla config set cloud-id 11111111-2222-3333-4444-555555555555` |
| `default-project` / `default_project` | active profile shorthand | Default Jira project for commands that can infer a project | `atla config set default-project PROJ` |
| `default-space` / `default_space` | active profile shorthand | Default Confluence space | `atla config set default-space ENG` |
| `profiles.<name>.instance` | profile-specific | Atlassian site URL for a named profile | `atla config set profiles.work.instance https://example.atlassian.net` |
| `profiles.<name>.email` | profile-specific | Email for a named profile | `atla config set profiles.work.email you@example.com` |
| `profiles.<name>.credential-store` / `profiles.<name>.credential_store` | profile-specific | Storage backend for a named profile | `atla config set profiles.work.credential-store keyring` |
| `profiles.<name>.cloud-id` / `profiles.<name>.cloud_id` | profile-specific | Cloud ID for product-specific scoped-token gateways | `atla config set profiles.work.cloud-id 11111111-2222-3333-4444-555555555555` |
| `profiles.<name>.policy.mode` | profile-specific | `read-only` or `read-write` default | `atla config set profiles.agent.policy.mode read-only` |
| `profiles.<name>.policy.allow` | profile-specific | Comma-separated allowed operation patterns | `atla config set profiles.agent.policy.allow jira.issue.view,jira.issue.comment.add` |
| `profiles.<name>.policy.deny` | profile-specific | Comma-separated denied patterns; highest priority | `atla config set profiles.agent.policy.deny '*.delete'` |
| `profiles.<name>.default-project` / `profiles.<name>.default_project` | profile-specific | Default Jira project for a named profile | `atla config set profiles.work.default-project PROJ` |
| `profiles.<name>.default-space` / `profiles.<name>.default_space` | profile-specific | Default Confluence space for a named profile | `atla config set profiles.work.default-space ENG` |
| `aliases.<name>` / `alias.<name>` | command alias | User-defined alias expanded before parsing | `atla config set aliases.mine "jira search 'assignee = currentUser()'"` |

Tokens are not config keys; they live in the OS keyring, the file credential store, or env vars.

## 8. Environment Variables

| Variable | Meaning | Default / precedence |
| --- | --- | --- |
| `ATLA_TOKEN` | Primary API token override | If set, used before stored credentials |
| `ATLA_API_TOKEN` | Alternate token override | Used if `ATLA_TOKEN` is unset |
| `ATLA_CONFIG` | Main config file path | Unix: `~/.config/atla/config.toml`; Windows: platform config directory |
| `ATLA_CREDENTIALS` | File credential store path | Unix: `~/.config/atla/credentials.toml`; Windows: platform config directory |
| `ATLA_READ_ONLY` | Enforce mutation blocking | Unset/false by default |

## 9. Common Patterns

### 1. List your open Jira work

```bash
atla --output json jira search 'assignee = currentUser() AND statusCategory != Done'   | jq '.issues[] | {key, summary: .fields.summary}'
```

### 2. Preview a Jira issue create in CI

```bash
atla --no-input --dry-run jira issue create --project PROJ --type Task --summary 'Release checklist'
```

### 3. Transition many issues from key-only output

```bash
atla jira issue list --project PROJ --status 'To Do' --output keys   | xargs -I{} atla --no-input jira issue transition {} --to 'In Progress'
```

### 4. Export sprint work to CSV

```bash
atla jira sprint issues 221 --fields summary,status,assignee,priority --output csv > sprint.csv
```

### 5. Create a Confluence page from Markdown

```bash
atla confluence page create --space ENG --title 'SSO Rollout'   --body-file docs/sso-rollout.md --representation markdown
# Add --numbered-table-rows for Confluence numbered table rows.
# Add --mention 'Name=ACCOUNT_ID' or --resolve-mentions for real ADF mentions.
```

### 6. Fetch Confluence page body as Markdown

```bash
atla confluence page view 123456 --format markdown
```

### 7. Copy a page template into another space

```bash
atla confluence page copy 123456 --title 'Incident Template' --space ENG --root-level
```

### 8. Search Confluence runbooks by label

```bash
atla confluence search 'type = page AND label = runbook AND space = ENG' --output json
```

### 9. Upload a release artifact to a page

```bash
atla confluence attachment upload 123456 ./artifacts/release-notes.pdf --comment 'Release package'
```

### 10. Delete safely with preview first

```bash
atla --dry-run confluence attachment delete 987654 --yes
atla confluence attachment delete 987654 --yes
```

### 11. Use an alternate profile for sandbox work

```bash
atla --profile sandbox jira project list
atla --profile sandbox confluence page list --space TEST
```

### 12. Run with isolated config in automation

```bash
ATLA_CONFIG=$PWD/.atla-config.toml ATLA_CREDENTIALS=$PWD/.atla-credentials.toml atla --no-input --output json config list
```

### 13. Store file-backed credentials for headless runs

```bash
printf '%s\n' "$ATLA_TOKEN" | atla auth login --no-input --storage file \
  --instance https://example.atlassian.net --email you@example.com --token-stdin
```
