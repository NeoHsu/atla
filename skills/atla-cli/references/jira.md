# Jira Command Reference

Complete syntax and flags for all `atla jira` commands. All commands accept global flags:
`-o/--output`, `--profile`, `--verbose`, `--dry-run`, `--no-input`.

**Pagination.** `--limit N` is a hard cap on returned items for the current invocation,
not a single-page hint. If more matches exist, `atla` returns an opaque `--page-token` for
the next logical page. Table output prints a ready-to-copy next command; JSON output
includes `pagination.nextPageToken` and `pagination.nextCommand`; CSV/keys keep stdout
record-only and write the next-page hint to stderr. Pass `--page-token` back to the same
command/query to continue. Use `--all` when you want every matching record; `--all` is
mutually exclusive with both `--limit` and `--page-token`. All paginating syntaxes below
also accept `--all` in place of `--limit`/`--page-token`.

---

## Projects

### List projects
```
atla jira project list [--query TEXT] [--limit N=50] [--page-token TOKEN]
```
Example: `atla jira project list --query platform --limit 25`

### View a project
```
atla jira project view <KEY>
```
Example: `atla jira project view PROJ`

### List issue types for a project
```
atla jira project issue-types <KEY>
```
Example: `atla jira project issue-types PROJ`

Use this before `issue create` to discover valid `--type` values.

---

## Search

### Run a JQL search
```
atla jira search <JQL> [--limit N=50] [--page-token TOKEN] [--fields FIELDS]
```
Example:
```bash
atla jira search 'project = PROJ AND statusCategory != Done ORDER BY updated DESC' \
  --limit 20 --fields summary,status,assignee,priority
```

---

## Issues

### List issues
```
atla jira issue list [--project KEY] [--status STATUS] [--type TYPE] [--assignee USER]
                     [--jql JQL] [--limit N=50] [--page-token TOKEN] [--fields FIELDS]
```
Example: `atla jira issue list --project PROJ --status 'In Progress' --assignee me`

### Create an issue
```
atla jira issue create --project KEY --type TYPE --summary TEXT
                       [--description TEXT | --description-file FILE]
                       [--field KEY=VALUE ...] [--labels LABELS]
```
Example:
```bash
atla jira issue create --project PROJ --type Task --summary 'Add SSO support' \
  --description-file docs/sso-plan.md \
  --field priority=High \
  --field customfield_10010='{"value":"Platform"}' \
  --labels auth,security
```

### Update an issue
```
atla jira issue update <KEY> [--summary TEXT] [--description TEXT | --description-file FILE]
                             [--field KEY=VALUE ...] [--labels LABELS]
```
Alias: `atla jira issue edit`

Labels support `add:` and `remove:` prefixes:
```bash
atla jira issue update PROJ-123 --labels add:urgent,remove:triage
```

### View an issue
```
atla jira issue view <KEY> [--web] [--fields FIELDS] [--with-github]
```
`--web` opens it in the browser. `--fields '*all'` fetches every field.
`--with-github` appends GitHub pull requests and commits from the development panel (auto-detects integration type).

### Delete an issue
```
atla jira issue delete <KEY> [--delete-subtasks] [--yes]
```

### Assign an issue
```
atla jira issue assign <KEY> <--to me|ACCOUNT_ID|NAME | --unassign> [--account-id]
```
Examples:
```bash
atla jira issue assign PROJ-123 --to me
atla jira issue assign PROJ-123 --to 5b10a2844c20165700ede21g --account-id
atla jira issue assign PROJ-123 --unassign
```

### Transition an issue
```
atla jira issue transition <KEY> [--to STATUS] [--field KEY=VALUE ...]
```
If `--to` is omitted and prompts are enabled, an interactive picker is shown.
Use `--no-input` in CI.
```bash
atla jira issue transition PROJ-123 --to Done
atla jira issue transition PROJ-123 --to 'In Review' --field resolution='{"name":"Done"}'
```

### List issue create-meta fields
```
atla jira issue fields --project KEY --type TYPE [--required-only]
```
Returns each field's ID, name, required flag, type, and allowed values.
Use this before `issue create` to discover what `--field` values are needed.

```bash
atla jira issue fields --project PROJ --type Bug --required-only
atla jira issue fields --project PROJ --type Bug -o json
```

---

## Comments

### Add a comment
```
atla jira issue comment add <KEY> [BODY | --body TEXT | --body-file FILE]
```

### List comments
```
atla jira issue comment list <KEY> [--limit N=25] [--page-token TOKEN]
```

### Update a comment
```
atla jira issue comment update <KEY> <COMMENT_ID> [--body TEXT | --body-file FILE]
```

### Delete a comment
```
atla jira issue comment delete <KEY> <COMMENT_ID> [--yes]
```

---

## Attachments

### Upload
```
atla jira issue attachment upload <KEY> --file FILE
```

### List
```
atla jira issue attachment list <KEY>
```

### Download
```
atla jira issue attachment download <KEY_OR_ID> [--all] [--dest PATH]
```
Use `--all` with an issue key to download all attachments.

### Delete
```
atla jira issue attachment delete <ATTACHMENT_ID> [--yes]
```

---

## Links

### Add an issue link
```
atla jira issue link add <KEY> --type TYPE --target KEY
```
Example: `atla jira issue link add PROJ-123 --type Blocks --target PROJ-456`

### List issue links
```
atla jira issue link list <KEY>
```

### Remove an issue link
```
atla jira issue link remove <LINK_ID> [--yes]
```

### GitHub development links

These commands read from Jira's internal dev-status API (`/rest/dev-status/1.0/issue/detail`), which powers the development panel in the Jira UI. The integration type is auto-detected from the summary endpoint — no manual configuration needed.

**Integration compatibility:**
- **Git Integration for Jira by GitKraken** (`oAuth-com.xiplink.jira.git.jira_git_plugin`) — surfaces both pull requests and commits; both `github-links` and `github-commits` work.
- **GitHub for Jira** (Atlassian native) — pull request data available via `github-links`.

#### List GitHub pull requests
```
atla jira issue link github-links <KEY>
```
Returns: status, PR id, author, source/destination branch, URL.
```bash
atla jira issue link github-links PROJ-123
atla jira issue link github-links PROJ-123 -o json
```

#### List GitHub commits
```
atla jira issue link github-commits <KEY>
```
Returns: short SHA, author, timestamp, repository name, commit message (first line), URL.
```bash
atla jira issue link github-commits PROJ-123
atla jira issue link github-commits PROJ-123 -o json | jq '.[].url'
```

---

## Worklogs

### Add a worklog
```
atla jira issue worklog add <KEY> --time TIME [--comment TEXT] [--started DATETIME]
```
Example: `atla jira issue worklog add PROJ-123 --time 1h30m --comment 'Debugged SSO' --started 2026-05-18T09:00:00Z`

### List worklogs
```
atla jira issue worklog list <KEY> [--limit N=25] [--page-token TOKEN]
```

---

## Boards

### List boards
```
atla jira board list [--project KEY] [--type TYPE] [--name NAME] [--limit N=50] [--page-token TOKEN]
```
Example: `atla jira board list --project PROJ --type scrum`

### View a board
```
atla jira board view <ID>
```

---

## Sprints

### List sprints
```
atla jira sprint list --board ID [--state STATE] [--limit N=50] [--page-token TOKEN]
```

### List active sprints
```
atla jira sprint active --board ID [--limit N=50] [--page-token TOKEN]
```

### View a sprint
```
atla jira sprint view <ID>
```

### Create a sprint
```
atla jira sprint create --board ID --name NAME [--start DATE] [--end DATE] [--goal TEXT]
```

### Start a sprint
```
atla jira sprint start <ID> [--start DATE] [--end DATE]
```

### Close a sprint
```
atla jira sprint close <ID>
```

### Add issues to a sprint
```
atla jira sprint add <ID> --issues KEY,KEY,...
# --issue is accepted as an alias for --issues
```

### Remove issues from a sprint
```
atla jira sprint remove <ID> --issues KEY,KEY,...
# --issue is accepted as an alias for --issues
```

### List sprint issues
```
atla jira sprint issues <ID> [--limit N=50] [--page-token TOKEN] [--fields FIELDS]
```

---

## JQL Quick Reference

| Goal | JQL |
|------|-----|
| Open issues in a project | `project = PROJ AND resolution = Unresolved` |
| Assigned to you | `assignee = currentUser() AND statusCategory != Done` |
| Recently updated | `updated >= -7d ORDER BY updated DESC` |
| Bugs only | `project = PROJ AND issuetype = Bug` |
| Filter by label | `labels = security` |
| Text search | `text ~ "single sign-on"` |
| Sprint backlog | `project = PROJ AND sprint = 221` |
| Created this week | `created >= startOfWeek()` |

---

## `--fields` Usage

Supported by: `search`, `issue list`, `issue view`, `sprint issues`.

Default fields: `summary,status,assignee,issuetype,priority`

```bash
atla jira issue list --project PROJ --fields summary,status,assignee,priority,labels
atla jira issue view PROJ-123 --fields '*all' --output json
atla jira search 'project = PROJ' --fields summary,status,customfield_10016
```

## `--field KEY=VALUE` on Mutating Commands

Available on `create`, `update`, `transition`. Sets arbitrary Jira fields.

- Raw JSON: `--field customfield_12345='{"value":"Ready"}'`
- Plain values auto-wrap: `--field priority=Highest` becomes `{"name":"Highest"}` — correct for option/priority fields
- `assignee=ID` becomes `{"accountId":"ID"}`
- `parent=PROJ-1` becomes `{"key":"PROJ-1"}`
- **String fields must use JSON string syntax**: `--field 'customfield_10166="5.1.0"'`
  A plain value like `5.1.0` is auto-wrapped as `{"name":"5.1.0"}`, which the API rejects for `string` type fields.
  Check the `type` column of `atla jira issue fields` to know when this applies.

Workflow: use `issue fields --required-only` first, then `issue create` with the required `--field` values:
```bash
atla jira issue fields --project PROJ --type Bug --required-only
atla jira issue create --project PROJ --type Bug --summary "Crash on login" \
  --field 'components=[{"id":"10582"}]' \
  --field 'customfield_10108={"id":"10022"}' \
  --field 'priority={"id":"10002"}' \
  --field 'customfield_10166="5.1.0"' \
  --field 'versions=[{"id":"13135"}]'
```
