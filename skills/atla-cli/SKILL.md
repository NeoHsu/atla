---
name: atla-cli
description: >
  Reference and execution skill for the `atla` Jira + Confluence Cloud CLI. Use it whenever
  the user wants to search, read, create, update, transition, attach, comment on, or delete
  Atlassian content; automate Jira/Confluence workflows; use JQL/CQL; or asks about an `atla`
  command. Trigger even when the user does not name the CLI, for requests such as “create a
  Jira ticket”, “check my sprint”, “publish this Confluence page”, 「開一張 Jira 票」、
  「建立工單」、「查 sprint」、「更新 Confluence 頁面」、「找我負責的 issue」、
  「搜尋 Confluence」或「上傳附件到頁面」, because `atla` is the installed execution tool.
---

# atla CLI

Use `atla` as a deterministic Jira + Confluence Cloud execution layer. Prefer machine-readable,
bounded, non-interactive commands and verify remote state after mutations.

## Execution gate

Run these gates before copying a command from the examples below.

### 1. Verify the active target

```bash
atla --output json auth status
```

Confirm the profile, instance, API target, and policy mode match the user's intended tenant. An
exit code `3` means auth/profile setup is missing; read `references/auth-config.md` and follow the
exact remediation from stderr before continuing. Never put a token in a shell argument when
`--token-stdin` is available.

### 2. Discover before mutating

Read the exact target with `--read-only`, use IDs returned by the API, and bound broad reads:

```bash
atla --read-only --max-pages 5 --max-items 200 --max-bytes 1000000 --timeout 30 \
  --output json confluence page view 123456
```

Record the target ID and current version. Do not infer a Jira project, Confluence space, profile,
or resource ID from a title alone.

### 3. Make mutation scope auditable

Before a mutation, state the operation, profile/site, target IDs, expected effect, and cleanup
method. Destructive commands require `--yes` and **never prompt**; without it they fail before
credentials or network access. Dry-run previews are exempt from confirmation.

For two or more live mutations, bulk work, or exhaustive smoke testing, keep an explicit temporary
resource ledger in the task notes or a private file. Record each operation, every created ID and
parent, and its cleanup state. Resume that ledger after a failure instead of restarting from
scratch, and only delete IDs created by the current run. If the user excludes a command or scope,
record it as `skipped` with a reason rather than silently counting it as passed.

### 4. Preview with the supported mechanism

Structured JSON dry-run and saved plans are supported only for:

- `jira.issue.create`
- `confluence.page.create`
- `confluence.page.update`
- `confluence.blog.create`
- `confluence.blog.update`

For these operations, prefer `atla plan ... --out FILE` followed by reviewed
`atla apply FILE --yes`. For any other operation, use a non-JSON `--dry-run`; combining an
unsupported operation with `--dry-run --output json` intentionally exits with code `2`.
`--read-only` permits stdout-only dry-run previews but blocks saved-plan files and apply.

### 5. Verify and clean up

After a mutation, query the target and compare the observed state with the requested state. On
`kind=ambiguous_mutation`, never repeat blindly: search the target first because the server may
have committed the request.

Normal Confluence page/blog/attachment deletion moves content to trash. `--purge` applies only to
an already-trashed item and requires space-admin permission; draft deletion is already permanent.
Keep cleanup failures and trash IDs visible in the result.

## Runtime source of truth

Use the references for command discovery and traps. Before relying on exact syntax, run:

```bash
atla <command path> --help
```

The runtime parser is authoritative. Repository examples are parse-checked, but parser checks do
not prove tenant permissions, API semantics, or cleanup success. For local discovery, use:

- `atla operation list --output json` for stable operation IDs and safety metadata;
- `atla schema list --output json` for bundled contracts;
- `atla explain-policy jira.issue.create --output json` for a profile decision.

## Global execution controls

| Flag | Contract |
|------|----------|
| `-o, --output json\|table\|csv\|keys` | Output format; use JSON for agents |
| `--profile NAME` | Select the exact auth/config profile |
| `--dry-run` | Preview without executing the mutation |
| `--read-only` | Reject local and remote writes |
| `--max-pages N` | Bound pages fetched during pagination |
| `--max-items N` | Bound accumulated records |
| `--max-bytes N` | Bound JSON output; requires `--output json` |
| `--timeout SECONDS` | Per-request timeout, including upload/download |
| `--no-input` | Disable interactive input for automation and CI |
| `--verbose` | Emit request/response diagnostics on stderr |

Runtime failures use exit codes `2` usage/policy, `3` auth, `4` not found, `5` safely retryable,
and `1` other. With JSON output, stderr contains a versioned error object. Stdout remains one JSON
document; warnings and pagination hints use stderr.

## Pagination

`--limit N` caps the current invocation. If more data exists, table output prints a next command,
JSON includes `pagination.nextPageToken` and `pagination.nextCommand`, and CSV/keys keep stdout
record-only while writing the hint to stderr. Tokens are opaque and query-bound.

`--all` is mutually exclusive with `--limit` and `--page-token`. Unbounded `--all` runs to
exhaustion; when global page/item budgets stop it, atla returns a resume token. Agents should
prefer bounded runs.

## Command tree

### Core

- `auth login/discover/logout/status/switch`
- `config set/get/list`
- `doctor [--network]`
- `explain-policy <OPERATION_ID>`
- `operation list`
- `schema list/print`
- `completion bash/elvish/fish/powershell/zsh`
- `plan jira ...` / `plan confluence ...`
- `apply <PLAN> --yes`

### Jira

- `project list/view/issue-types`
- `search <JQL>`
- `issue list/create/view/update/edit/delete/fields/assign/transition`
- `issue comment add/list/update/delete`
- `issue attachment upload/list/download/delete`
- `issue link add/list/remove/github-links/github-commits`
- `issue worklog add/list`
- `board list/view`
- `sprint list/active/view/create/start/close/add/remove/issues`

### Confluence

- `space list/view/create/update/delete`
- `page list/view/children/copy/create/update/move/delete`
- `page label list/add/remove`
- `page comment list/add/delete`
- `blog list/view/create/update/delete`
- `blog label list/add/remove`
- `blog comment list/add/delete`
- `search <CQL>`
- `attachment list/view/upload/download/delete`

## Quick patterns

### Find open work

```bash
atla --read-only --output json jira search \
  'assignee = currentUser() AND statusCategory != Done ORDER BY updated DESC' --limit 50
```

### Discover required fields, then create

```bash
atla jira issue fields --project APP --type Task --required-only
atla jira issue create --project APP --type Task --summary 'Fix login failure'
```

String custom fields require JSON string syntax, for example
`--field 'customfield_10166="5.1.0"'`; read `references/jira.md` before supplying arbitrary fields.

### Read or author Confluence Markdown

```bash
atla --read-only confluence page view 123456 --format markdown
atla confluence page create --space ENG --title 'Meeting Notes' \
  --body-file notes.md --representation markdown --parent 654321
```

### Comment with an attachment

```bash
atla confluence page comment add 123456 'See attached evidence' \
  --attachment ./evidence.png --attachment-mode auto
```

Confluence-hosted files must be uploaded as attachments before they are referenced. See the
storage/ADF examples and CSP caveat in `references/confluence.md` rather than embedding an external
URL.

## Common traps

- `confluence page view` and `blog view` return metadata unless `--format` is supplied.
- Page/blog create/update and page/blog comments default body input to `storage`; always pass
  `--representation markdown` for Markdown files. Markdown input is converted to ADF.
- Page/blog update plans are offline: supply explicit title, body/body-file, and next version.
  Create plans that name a Confluence space must use `--space-id`.
- Confluence footer-comment replies must identify the parent comment; do not reuse an unrelated
  page/blog comment ID.
- Confluence `--purge` is a second delete after trashing and requires space-admin permission.
- Jira transitions may prompt only when `--to` is omitted and both stdin/stdout are TTYs. Use
  `--no-input` for agents.
- Treat `--page-token` and plan hashes as opaque. A plan digest detects modification but is not a
  signature; never apply an untrusted plan.
- `doctor` is local-only unless `--network` is explicit. It reports token availability/source but
  never prints the token. `schema print` supports default/JSON output, not table/csv/keys.

## Load the relevant reference

Read only the file needed for the task:

- `references/jira.md` — exact Jira syntax, fields, JQL, boards, and sprints
- `references/confluence.md` — exact Confluence syntax, representations, CQL, and attachments
- `references/auth-config.md` — login, discovery, profiles, policy, aliases, and environment
- `references/plans.md` — supported plan/apply operations and validation requirements
