---
title: Jira Reference
description: Complete Jira command reference for atla.
---

# Jira

`atla jira` covers project discovery, JQL search, issue workflows, boards, and sprints.
All commands also accept the global flags described in [`output-formats.md`](./output-formats.md):
`-o, --output`, `--profile`, `--verbose`, `--dry-run`, and `--no-input`.

## Pagination

Every `--limit N` flag is a "max-results" cap, not a single-page hint. `atla` paginates
the underlying Atlassian API internally and accumulates up to `N` items before returning,
so `--limit 5000` reliably returns 5000 results (subject to data volume) rather than the
~100 a single Jira page would yield.

If the server still has more matches when the limit is hit, `atla` returns a `--page-token`
for the next logical page instead of forcing you to increase `--limit`. In table output,
the token is shown as a ready-to-copy command:

```text
More results available.
Next page:
  atla jira search 'project = PROJ' --limit 50 --page-token <TOKEN>
```

In JSON output, pagination metadata is included alongside the records:

```json
{
  "issues": [],
  "pagination": {
    "isLast": false,
    "nextPageToken": "...",
    "nextCommand": "atla jira search 'project = PROJ' --limit 50 --page-token ..."
  }
}
```

For `csv` and `keys` output, records stay on stdout and the next-page hint is written to
stderr so pipelines remain clean. `--page-token` is intentionally opaque; pass it back to
the same command/query to continue. Tokens are validated against the command and query,
and using one with a different query fails fast.

### `--all`

When you want every matching record without guessing an upper bound, use `--all`. It
fetches until the server reports no more results, ignores the `--limit` clamp, and
does not emit next-page metadata because it fetches until exhaustion:

```bash
atla jira issue list --jql "project = PROJ" --all --output keys > all-keys.txt
atla jira board list --all --output json | jq '.values | length'
```

`--all` is mutually exclusive with both `--limit` and `--page-token`. Be aware that
`--all` against a large result set will issue many HTTP requests (one per 100 items for
JQL, one per 100 for agile / v2 endpoints), so use it deliberately on broad queries.

### Affected commands

`jira project list`, `jira search`, `jira issue list`, `jira issue comment list`,
`jira issue worklog list`, `jira board list`, `jira sprint list`, `jira sprint active`,
and `jira sprint issues`.

## Projects

### List projects

**Syntax**

```bash
atla jira project list [--query TEXT] [--limit N=50] [--page-token TOKEN]
```

**Example**

```bash
atla jira project list --query platform --limit 25
```

### View a project

**Syntax**

```bash
atla jira project view <KEY>
```

**Example**

```bash
atla jira project view PROJ
```

### List issue types for a project

**Syntax**

```bash
atla jira project issue-types <KEY>
```

**Example**

```bash
atla jira project issue-types PROJ
```

## Search

### Run a JQL search

**Syntax**

```bash
atla jira search <JQL> [--limit N=50] [--page-token TOKEN] [--fields FIELDS]
```

**Example**

```bash
atla jira search 'project = PROJ AND statusCategory != Done ORDER BY updated DESC'   --limit 20   --fields summary,status,assignee,priority
```

## Issues

### List issues

**Syntax**

```bash
atla jira issue list [--project KEY] [--status STATUS] [--type TYPE] [--assignee USER]                      [--jql JQL] [--limit N=50] [--page-token TOKEN] [--fields FIELDS]
```

**Example**

```bash
atla jira issue list --project PROJ --status 'In Progress' --assignee me --limit 25
```

### Create an issue

**Syntax**

```bash
atla jira issue create --project KEY --type TYPE --summary TEXT                        [--description TEXT | --description-file FILE]                        [--field KEY=VALUE ...] [--labels LABELS]
```

**Example**

```bash
atla jira issue create --project PROJ --type Task --summary 'Add SSO support'   --description-file docs/sso-plan.md   --field priority=High   --field customfield_10010='{"value":"Platform"}'   --labels auth,security
```

### Update an issue

**Syntax**

```bash
atla jira issue update <KEY> [--summary TEXT] [--description TEXT | --description-file FILE]                              [--field KEY=VALUE ...] [--labels LABELS]
```

Alias: `atla jira issue edit`

**Example**

```bash
atla jira issue update PROJ-123 --summary 'Add SSO support to admin login'   --labels add:urgent,remove:triage
```

### View an issue

**Syntax**

```bash
atla jira issue view <KEY> [--web] [--fields FIELDS] [--with-github]
```

**Examples**

```bash
atla jira issue view PROJ-123
atla jira issue view PROJ-123 --web
atla jira issue view PROJ-123 --with-github
```

`--with-github` fetches GitHub pull requests and commits linked via the development panel and appends them to the output. See [GitHub Development Links](#github-development-links) for details.

### Delete an issue

**Syntax**

```bash
atla jira issue delete <KEY> [--delete-subtasks] [--yes]
```

**Example**

```bash
atla jira issue delete PROJ-123 --delete-subtasks --yes
```

### Assign an issue

**Syntax**

```bash
atla jira issue assign <KEY> <--to me|ACCOUNT_ID|NAME | --unassign> [--account-id]
```

**Examples**

```bash
atla jira issue assign PROJ-123 --to me
atla jira issue assign PROJ-123 --to 5b10a2844c20165700ede21g --account-id
atla jira issue assign PROJ-123 --unassign
```

### Transition an issue

**Syntax**

```bash
atla jira issue transition <KEY> [--to STATUS] [--field KEY=VALUE ...]
```

**Examples**

```bash
atla jira issue transition PROJ-123 --to Done
atla jira issue transition PROJ-123 --to 'In Review'   --field resolution='{"name":"Done"}'
```

If `--to` is omitted and prompts are enabled, `atla` can offer an interactive transition picker.
Use `--no-input` in CI to disable that prompt.

### Comments

#### Add a comment

**Syntax**

```bash
atla jira issue comment add <KEY> [BODY | --body TEXT | --body-file FILE]
                                  [--attachment FILE ...]
                                  [--attachment-mode auto|link|embed]
```

**Example**

```bash
atla jira issue comment add PROJ-123 --body 'Ready for review.'
atla jira issue comment add PROJ-123 --body 'Please check the logs.'   --attachment ./error.log --attachment ./screenshot.png
```

When `--attachment` is used, `atla` uploads each file to the issue first, then appends attachment references to the new comment. `auto` embeds image-style references for images and links other files; use `link` for links only or `embed` to request image-style references for images.

#### List comments

**Syntax**

```bash
atla jira issue comment list <KEY> [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla jira issue comment list PROJ-123 --limit 10
```

#### Update a comment

**Syntax**

```bash
atla jira issue comment update <KEY> <COMMENT_ID> [--body TEXT | --body-file FILE]
```

**Example**

```bash
atla jira issue comment update PROJ-123 10001 --body 'QA passed. Merging now.'
```

#### Delete a comment

**Syntax**

```bash
atla jira issue comment delete <KEY> <COMMENT_ID> [--yes]
```

**Example**

```bash
atla jira issue comment delete PROJ-123 10001 --yes
```

### Attachments

#### Upload an attachment

**Syntax**

```bash
atla jira issue attachment upload <KEY> --file FILE
```

**Example**

```bash
atla jira issue attachment upload PROJ-123 --file ./screenshots/login-error.png
```

#### List attachments

**Syntax**

```bash
atla jira issue attachment list <KEY>
```

**Example**

```bash
atla jira issue attachment list PROJ-123
```

#### Download attachments

**Syntax**

```bash
atla jira issue attachment download <KEY_OR_ID> [--all] [--dest PATH]
```

**Examples**

```bash
atla jira issue attachment download 10020 --dest ./downloads
atla jira issue attachment download PROJ-123 --all --dest ./downloads
```

#### Delete an attachment

**Syntax**

```bash
atla jira issue attachment delete <ATTACHMENT_ID> [--yes]
```

**Example**

```bash
atla jira issue attachment delete 10020 --yes
```

### Links

#### Add an issue link

**Syntax**

```bash
atla jira issue link add <KEY> --type TYPE --target KEY
```

**Example**

```bash
atla jira issue link add PROJ-123 --type Blocks --target PROJ-456
```

#### List issue links

**Syntax**

```bash
atla jira issue link list <KEY>
```

**Example**

```bash
atla jira issue link list PROJ-123
```

#### Remove an issue link

**Syntax**

```bash
atla jira issue link remove <LINK_ID> [--yes]
```

**Example**

```bash
atla jira issue link remove 10500 --yes
```

### GitHub Development Links

These commands fetch development data from Jira's internal dev-status API (`/rest/dev-status/1.0/issue/detail`). The API is not publicly documented by Atlassian, but is used internally by the Jira UI to render the development panel.

> **Integration note:** The available data depends on which Git integration your Jira instance uses:
> - **Git Integration for Jira by GitKraken** (`oAuth-com.xiplink.jira.git.jira_git_plugin`) — surfaces both pull requests and commits. Both `github-links` and `github-commits` work.
> - **GitHub for Jira** (Atlassian's native app) — surfaces pull requests directly.
>
> Both `github-links` and `github-commits` auto-detect the active integration from the summary endpoint so no manual configuration is needed.

#### List GitHub pull requests

**Syntax**

```bash
atla jira issue link github-links <KEY>
```

**Example**

```bash
atla jira issue link github-links PROJ-123
atla jira issue link github-links PROJ-123 -o json
```

Returns: `status`, `id`, `title`, `author`, `source` branch, `destination` branch, `url`.

#### List GitHub commits

**Syntax**

```bash
atla jira issue link github-commits <KEY>
```

**Example**

```bash
atla jira issue link github-commits PROJ-123
atla jira issue link github-commits PROJ-123 -o json | jq '.[].url'
```

Returns: `id` (short SHA), `author`, `timestamp`, `repository`, `message` (first line), `url`.

### Issue Fields

List the fields available when creating an issue, including which are required and what values are allowed.

**Syntax**

```bash
atla jira issue fields --project KEY --type TYPE [--required-only]
```

**Flags**

| Flag | Description |
| --- | --- |
| `--project KEY` | Project key (required) |
| `--type TYPE` | Issue type name or ID — e.g. `Bug`, `Story`, `Task` (required) |
| `--required-only` | Show only required fields |

**Examples**

```bash
# All fields for Bug in PROJ
atla jira issue fields --project PROJ --type Bug

# Only the required fields
atla jira issue fields --project PROJ --type Bug --required-only

# Machine-readable output for scripting
atla jira issue fields --project PROJ --type Bug -o json
```

**Output columns**

| Column | Description |
| --- | --- |
| `field_id` | Field ID to use in `--field` (e.g. `customfield_10108`) |
| `name` | Human-readable field name (e.g. `Severity`) |
| `required` | `true` if the field must be provided on create |
| `type` | Schema type: `string`, `option`, `array`, `priority`, `user`, etc. |
| `allowed_values` | First 5 allowed values for option/array fields |

**Typical workflow — create with all required fields**

```bash
# 1. Discover required fields
atla jira issue fields --project PROJ --type Bug --required-only

# 2. Create with the required fields filled in
atla jira issue create --project PROJ --type Bug \
  --summary "Login page crashes on Safari" \
  --field 'components=[{"id":"10582"}]' \
  --field 'customfield_10108={"id":"10022"}' \
  --field 'priority={"id":"10002"}' \
  --field 'customfield_10166="5.1.0"' \
  --field 'versions=[{"id":"13135"}]'
```

> **Tip:** For `string` type fields (e.g. `Affect Build`), wrap the value in JSON string quotes:
> `--field 'customfield_10166="5.1.0"'` — not `--field customfield_10166=5.1.0`.
> Plain values without quotes are auto-wrapped as `{"name":"..."}`, which the API rejects for string fields.

### Worklogs

#### Add a worklog

**Syntax**

```bash
atla jira issue worklog add <KEY> --time TIME [--comment TEXT] [--started DATETIME]
```

**Example**

```bash
atla jira issue worklog add PROJ-123 --time 1h30m   --comment 'Investigated SSO callback failures'   --started 2026-05-18T09:00:00Z
```

#### List worklogs

**Syntax**

```bash
atla jira issue worklog list <KEY> [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla jira issue worklog list PROJ-123 --limit 10
```

## Boards

### List boards

**Syntax**

```bash
atla jira board list [--project KEY] [--type TYPE] [--name NAME] [--limit N=50] [--page-token TOKEN]
```

**Example**

```bash
atla jira board list --project PROJ --type scrum --limit 10
```

### View a board

**Syntax**

```bash
atla jira board view <ID>
```

**Example**

```bash
atla jira board view 84
```

## Sprints

### List sprints

**Syntax**

```bash
atla jira sprint list --board ID [--state STATE] [--limit N=50] [--page-token TOKEN]
```

**Example**

```bash
atla jira sprint list --board 84 --state active --limit 10
```

### List active sprints

**Syntax**

```bash
atla jira sprint active --board ID [--limit N=50] [--page-token TOKEN]
```

**Example**

```bash
atla jira sprint active --board 84
```

### View a sprint

**Syntax**

```bash
atla jira sprint view <ID>
```

**Example**

```bash
atla jira sprint view 221
```

### Create a sprint

**Syntax**

```bash
atla jira sprint create --board ID --name NAME [--start DATE] [--end DATE] [--goal TEXT]
```

**Example**

```bash
atla jira sprint create --board 84 --name 'Sprint 42'   --start 2026-05-20 --end 2026-06-02 --goal 'Ship SSO MVP'
```

### Start a sprint

**Syntax**

```bash
atla jira sprint start <ID> [--start DATE] [--end DATE]
```

**Example**

```bash
atla jira sprint start 221 --start 2026-05-20 --end 2026-06-02
```

### Close a sprint

**Syntax**

```bash
atla jira sprint close <ID>
```

**Example**

```bash
atla jira sprint close 221
```

### Add issues to a sprint

**Syntax**

```bash
atla jira sprint add <ID> --issues KEY,KEY,...
```

Alias: `--issue`

**Example**

```bash
atla jira sprint add 221 --issues PROJ-123,PROJ-124,PROJ-130
```

### Remove issues from a sprint

**Syntax**

```bash
atla jira sprint remove <ID> --issues KEY,KEY,...
```

Alias: `--issue`

**Example**

```bash
atla jira sprint remove 221 --issues PROJ-130
```

### List sprint issues

**Syntax**

```bash
atla jira sprint issues <ID> [--limit N=50] [--page-token TOKEN] [--fields FIELDS]
```

**Example**

```bash
atla jira sprint issues 221 --limit 50 --fields summary,status,assignee,storyPoints
```

## JQL quick reference

JQL is available in `atla jira search` and in `atla jira issue list --jql ...`.

| Goal | Example |
| --- | --- |
| Open issues in one project | `project = PROJ AND resolution = Unresolved` |
| Assigned to you | `assignee = currentUser() AND statusCategory != Done` |
| Recently updated | `updated >= -7d ORDER BY updated DESC` |
| Bugs only | `project = PROJ AND issuetype = Bug` |
| Filter by label | `labels = security` |
| Search by text | `text ~ "single sign-on"` |
| Sprint backlog | `project = PROJ AND sprint = 221` |
| Created this week | `created >= startOfWeek()` |

Examples:

```bash
atla jira search 'assignee = currentUser() AND statusCategory != Done ORDER BY priority DESC'
atla jira issue list --jql 'project = PROJ AND labels = security' --fields summary,status,labels
```

## Customizing output with `--fields`

`--fields` is supported by:

- `atla jira search`
- `atla jira issue list`
- `atla jira issue view`
- `atla jira sprint issues`

Use a comma-separated list of Jira field names:

```bash
atla jira issue list --project PROJ --fields summary,status,assignee,priority,labels
```

### Default field set

If you omit `--fields`, `atla` requests:

```text
summary,status,assignee,issuetype,priority
```

### Useful patterns

```bash
# Include custom fields
atla jira search 'project = PROJ' --fields summary,status,customfield_10016

# Fetch everything the API returns
atla jira issue view PROJ-123 --fields '*all' --output json
```

### `--field KEY=VALUE` on mutating commands

On `create`, `update`, and `transition`, `--field` lets you set arbitrary Jira fields.

- Raw JSON is accepted: `--field customfield_12345='{"value":"Ready"}'`
- Plain values are auto-wrapped as `{"name":"VALUE"}` — correct for option/priority fields
- `assignee=...` becomes `{"accountId":"..."}`
- `parent=PROJ-1` becomes `{"key":"PROJ-1"}`
- **Text (string) fields must be quoted as a JSON string**: `--field customfield_10166='"5.1.0"'`
  Plain values like `5.1.0` would be auto-wrapped as `{"name":"5.1.0"}`, which the API rejects for string fields.

Examples:

```bash
atla jira issue create --project PROJ --type Story --summary 'Parent issue' \
  --field priority=Highest

atla jira issue transition PROJ-123 --to Done \
  --field resolution='{"name":"Done"}'

# String custom field — use JSON string syntax
atla jira issue create --project PROJ --type Bug --summary 'Login crash' \
  --field 'customfield_10166="5.1.0"'
```

### Discovering required fields

Before creating an issue in a project that has required custom fields, use `issue fields` to see what the API expects:

```bash
atla jira issue fields --project PROJ --type Bug --required-only
```

## `--dry-run` tips

`--dry-run` is available globally and is especially useful for write operations.

```bash
atla --dry-run jira issue create --project PROJ --type Task --summary 'Validate release checklist'
atla --dry-run jira issue delete PROJ-123 --yes
atla --dry-run jira sprint start 221 --start 2026-05-20 --end 2026-06-02
```

What it does:

- Prints the Jira REST call `atla` would make.
- Skips the actual API request.
- Lets you validate flags, IDs, and target profile before a destructive change.
- Works well with `--no-input` for automation and CI.
