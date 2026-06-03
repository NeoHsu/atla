# Bulk Issue Operations

This document specifies the design and implementation plan for adding JQL-based bulk
operations to `atla jira issue`. The goal is to let coding agents (and power users) act
on many issues in a single command, without shell loops or multiple CLI invocations.

---

## Motivation

A coding agent doing sprint management today must:

```bash
# 3 steps, N API round-trips
keys=$(atla jira issue list --jql "project=X AND status='To Do'" -o keys)
for key in $keys; do
  atla jira issue transition "$key" --to "In Progress"
done
```

After this change it becomes one command, one bulk API call, and one async poll:

```bash
atla jira issue transition --jql "project=X AND status='To Do'" --to "In Progress"
```

---

## Scope

| Command | Mode | Jira API |
|---------|------|----------|
| `issue transition` | `--jql` | `POST /rest/api/3/bulk/issues/transition` (async) |
| `issue delete` | `--jql` | `POST /rest/api/3/bulk/issues/delete` (async) |
| `issue create` | `--from-file` | `POST /rest/api/3/issue/bulk` (sync, ≤50/batch) |

`issue update --jql` is **out of scope** for this iteration. The Jira bulk edit API
(`POST /rest/api/3/bulk/issues/fields`) requires a complex typed field schema
(`JiraIssueFields`) that doesn't map cleanly to the existing `--field KEY=VALUE` flags.

---

## CLI Interface

### `issue transition`

```
atla jira issue transition [KEY] [--jql <JQL>] --to <transition> [--no-notify] [--dry-run]
```

`KEY` and `--jql` are mutually exclusive. Exactly one is required (enforced by clap arg group).

| Flag | Description |
|------|-------------|
| `KEY` | Single issue key (existing behaviour) |
| `--jql <JQL>` | Select issues via JQL query |
| `--to <name>` | Transition name or ID (required) |
| `--no-notify` | Suppress bulk-change email notification |

**Example – single issue (unchanged):**
```bash
atla jira issue transition PROJ-123 --to Done
```

**Example – bulk via JQL:**
```bash
atla jira issue transition --jql "project=PROJ AND sprint in openSprints() AND status='To Do'" \
  --to "In Progress"
```

**Example – dry-run:**
```bash
atla jira issue transition --jql "project=PROJ AND status=Open" --to Done --dry-run
# Would search JQL then POST /rest/api/3/bulk/issues/transition for N issues
```

---

### `issue delete`

```
atla jira issue delete [KEY] [--jql <JQL>] --yes [--no-notify] [--dry-run]
```

| Flag | Description |
|------|-------------|
| `KEY` | Single issue key (existing behaviour) |
| `--jql <JQL>` | Select issues via JQL query |
| `--yes` | Required to confirm destructive operation |
| `--no-notify` | Suppress bulk-change email notification |

**Example:**
```bash
atla jira issue delete --jql "project=OLD AND created < -365d" --yes
```

---

### `issue create`

```
atla jira issue create [--project P --type T --summary S …] [--from-file <path>]
```

`--from-file` and the individual field flags (`--project`, `--type`, `--summary`, etc.)
are mutually exclusive (enforced by clap arg group or runtime check).

**Input file format** (`tasks.json`):

```json
[
  {
    "project": "PROJ",
    "type": "Task",
    "summary": "Fix login bug",
    "description": "Users see 500 on /login",
    "labels": ["bug", "urgent"]
  },
  {
    "project": "PROJ",
    "type": "Story",
    "summary": "OAuth implementation",
    "fields": {
      "priority": { "name": "High" }
    }
  }
]
```

The top-level keys `project`, `type`, `summary`, `description`, and `labels` mirror the
existing single-issue flags. The optional `fields` object accepts raw Jira field values
and is merged in last (same semantics as `--field KEY=VALUE` today).

**Example:**
```bash
atla jira issue create --from-file tasks.json
```

---

## Output

### Table mode (default)

```
Searching issues…  47 found
Resolving transitions…
Submitting bulk transition → "In Progress"
Progress:  65% [=====================>         ] (task 10641)
Progress: 100% [==============================] done

  47 transitioned  ·  0 failed  ·  0 inaccessible
```

When there are failures, list them:

```
  45 transitioned  ·  2 failed  ·  0 inaccessible
  PROJ-12  workflow requires a field update (use Jira UI for bulk transitions with fields)
  PROJ-34  permission denied
```

### JSON mode (`-o json`)

```json
{
  "task_id": "10641",
  "status": "COMPLETE",
  "total": 47,
  "processed": 47,
  "failed": 0,
  "inaccessible": 0,
  "failed_issues": {}
}
```

### Keys mode (`-o keys`)

Outputs only the successfully processed issue keys (one per line), suitable for piping.

---

## Implementation Plan

### Layer 1 — Core models (`atla-core/src/jira/models.rs`)

Add the following types:

```rust
// Lightweight issue reference returned by key searches
pub struct JiraIssueRef {
    pub id: String,
    pub key: String,
}

// Request — JQL-based bulk transition
pub struct JiraIssueBulkTransition {
    pub jql: String,
    pub to: String,           // transition name (resolved to ID internally)
    pub max_results: u32,     // max issues to match, clamped to 1000
    pub send_notification: bool,
}

// Request — JQL-based bulk delete
pub struct JiraIssueBulkDelete {
    pub jql: String,
    pub max_results: u32,
    pub send_notification: bool,
}

// Intermediate — transition lookup grouped by workflow
pub struct BulkWorkflowTransitions {
    pub groups: Vec<BulkWorkflowGroup>,
}

pub struct BulkWorkflowGroup {
    pub issue_keys: Vec<String>,
    pub transitions: Vec<BulkAvailableTransition>,
}

pub struct BulkAvailableTransition {
    pub id: String,
    pub name: String,
    pub to_status: String,
}

// Response — async task submitted
pub struct BulkTaskSubmitted {
    pub task_id: String,
}

// Response — async task progress / result
pub struct BulkTaskResult {
    pub task_id: String,
    pub status: BulkTaskStatus,
    pub progress_percent: u8,
    pub total: u32,
    pub processed: u32,
    pub failed: u32,
    pub inaccessible: u32,
    pub failed_issues: HashMap<String, Vec<String>>,  // issue_id → reasons
}

pub enum BulkTaskStatus {
    Enqueued,
    Running,
    Complete,
    Failed,
    Cancelled,
    Dead,
}

impl BulkTaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Failed | Self::Cancelled | Self::Dead)
    }
}

// Response — bulk create
pub struct BulkCreateResult {
    pub created: Vec<JiraCreatedIssue>,
    pub errors: Vec<BulkCreateError>,
}

pub struct BulkCreateError {
    pub index: usize,
    pub messages: Vec<String>,
}
```

---

### Layer 2 — API client (`atla-core/src/jira/issues.rs`)

Add the following methods to `JiraClient`:

#### `search_issue_keys`

```rust
pub async fn search_issue_keys(
    &self,
    jql: &str,
    max_results: u32,
) -> Result<Vec<JiraIssueRef>, ApiError>
```

Calls `search_and_reconcile_issues_using_jql` with `fields=["key"]` only. Returns
`JiraIssueRef` pairs (both `id` and `key` are present in every search response at no
extra cost). The `id→key` map built from this result is passed through to
`print_bulk_result` to support `-o keys` output, since the bulk queue response returns
integer issue IDs rather than keys.

#### `get_bulk_transitions`

```rust
pub async fn get_bulk_transitions(
    &self,
    issue_keys: &[String],
) -> Result<BulkWorkflowTransitions, ApiError>
```

Calls `GET /rest/api/3/bulk/issues/transition?issueIdsOrKeys=KEY1,KEY2,…` via
`self.generated.get_available_transitions()` (operationId `getAvailableTransitions` from
the partial spec). The `issueIdsOrKeys` query parameter is a comma-separated key list
which is the full batch (≤1000) — no pagination needed for our use case since we control
the input size. Convert the returned `BulkTransitionGetAvailableTransitions` into the
local `BulkWorkflowTransitions` model.

#### `submit_bulk_transition`

```rust
pub async fn submit_bulk_transition(
    &self,
    inputs: Vec<(Vec<String>, String)>,  // (issue_keys, transition_id)
    send_notification: bool,
) -> Result<BulkTaskSubmitted, ApiError>
```

Calls `POST /rest/api/3/bulk/issues/transition` via `self.generated.submit_bulk_transition()`
(operationId `submitBulkTransition`). Accepts pre-resolved `(issue_keys, transition_id)`
pairs, builds the `IssueBulkTransitionPayload` body (a `bulkTransitionInputs` array of
`BulkTransitionSubmitInput` plus `sendBulkNotification`), and maps the returned
`SubmittedBulkOperation` (`{"taskId": "..."}`) into `BulkTaskSubmitted`.

#### `submit_bulk_delete`

```rust
pub async fn submit_bulk_delete(
    &self,
    issue_keys: Vec<String>,
    send_notification: bool,
) -> Result<BulkTaskSubmitted, ApiError>
```

Calls `POST /rest/api/3/bulk/issues/delete` via `self.generated.submit_bulk_delete()`
(operationId `submitBulkDelete`). Builds the `IssueBulkDeletePayload` body
(`selectedIssueIdsOrKeys` + `sendBulkNotification`) and maps `SubmittedBulkOperation` into
`BulkTaskSubmitted`.

#### `poll_bulk_task`

```rust
pub async fn poll_bulk_task(
    &self,
    task_id: &str,
) -> Result<BulkTaskResult, ApiError>
```

Calls `GET /rest/api/3/bulk/queue/{taskId}` via
`self.generated.get_bulk_operation_progress()` (operationId `getBulkOperationProgress`).
Maps the returned `BulkOperationProgress` into the local `BulkTaskResult`. Returns the
current snapshot; callers are responsible for polling.

#### `bulk_create_issues`

```rust
pub async fn bulk_create_issues(
    &self,
    issues: &[JiraIssueCreate],
) -> Result<BulkCreateResult, ApiError>
```

Calls `POST /rest/api/3/issue/bulk` via `self.generated.create_issues()` (operationId
`createIssues`). The Jira API accepts ≤50 issues per call. Internally, chunk `issues`
into batches of 50, call the endpoint per batch, and aggregate `CreatedIssue` and
`BulkOperationErrorResult` entries from the returned `CreatedIssues` into
`BulkCreateResult`. Batches are called **sequentially** (not concurrently) to stay within
Jira's rate limits.

**Note on payload construction:** the existing `JiraIssueCreate.to_generated()` method
produces a single-issue `IssueUpdateDetails`, but the bulk endpoint takes an
`IssuesUpdateBean` whose wire format is `{"issueUpdates": [{"fields": {...}, "update":
{...}}, ...]}` — an array of `IssueUpdateDetails`. `bulk_create_issues` must build that
array by calling `to_generated()` on each `JiraIssueCreate` and collecting the results
into `IssuesUpdateBean.issueUpdates` (or introduce a small helper such as
`JiraIssueCreate::to_generated_bulk(issues: &[JiraIssueCreate]) -> IssuesUpdateBean`);
do **not** pass `to_generated()` directly as the request body.

---

### Layer 3 — CLI argument definitions (`atla-cli/src/cli.rs`)

#### `IssueAction::Transition`

Change from a required positional `key: String` to an arg group:

```rust
Transition {
    #[command(flatten)]
    target: IssueTransitionTargetArgs,
    #[arg(long)]
    to: Option<String>,
    #[arg(long = "field")]
    fields: Vec<String>,
    #[arg(long)]
    no_notify: bool,
},

#[derive(Debug, Args)]
#[group(id = "transition_target", required = true, multiple = false)]
pub struct IssueTransitionTargetArgs {
    /// Single issue key (e.g. PROJ-123)
    pub key: Option<String>,
    /// JQL query selecting the issues to transition
    #[arg(long)]
    pub jql: Option<String>,
}
```

`--to` remains optional for single-issue mode (interactive prompt fallback), but is
**required** when `--jql` is used — enforced at runtime in the handler with a clear error
message.

`--field` is only meaningful in single-issue mode. In `--jql` mode the handler must
`bail!` with `"--field is not supported in --jql mode; bulk transitions cannot apply field updates"`
if any `--field` flags are provided, since the Jira bulk transition API excludes
transitions that require field updates.

#### `IssueAction::Delete`

Same pattern:

```rust
Delete {
    #[command(flatten)]
    target: IssueDeleteTargetArgs,
    #[arg(long)]
    delete_subtasks: bool,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    no_notify: bool,
},

#[derive(Debug, Args)]
#[group(id = "delete_target", required = true, multiple = false)]
pub struct IssueDeleteTargetArgs {
    pub key: Option<String>,
    #[arg(long)]
    pub jql: Option<String>,
}
```

`--delete-subtasks` is only meaningful in single-issue mode. The Jira bulk delete API
(`POST /rest/api/3/bulk/issues/delete`) has no equivalent parameter. In `--jql` mode the
handler must `bail!` if `--delete-subtasks` is provided.

#### `IssueAction::Create`

```rust
Create {
    // existing flags become optional when --from-file is provided
    #[arg(long, required_unless_present = "from_file")]
    project: Option<String>,
    #[arg(long = "type", required_unless_present = "from_file")]
    issue_type: Option<String>,
    #[arg(long, required_unless_present = "from_file")]
    summary: Option<String>,
    #[arg(long, conflicts_with = "description_file")]
    description: Option<String>,
    #[arg(long)]
    description_file: Option<PathBuf>,
    #[arg(long = "field")]
    fields: Vec<String>,
    #[arg(long)]
    labels: Option<String>,
    /// Path to a JSON file containing an array of issue objects for bulk creation
    #[arg(long, conflicts_with_all = ["project", "issue_type", "summary"])]
    from_file: Option<PathBuf>,
},
```

---

### Layer 4 — Command handlers (`atla-cli/src/commands/jira/issue.rs`)

#### Bulk transition handler

```
1. jql branch detected (target.jql.is_some())
2. Require --to (bail if missing)
3. Search issue keys: client.search_issue_keys(jql, max_results=1000)
4. Bail if 0 results
5. --dry-run: print intent and return
6. client.get_bulk_transitions(&keys) → BulkWorkflowTransitions
7. For each workflow group, find transition matching --to name (case-insensitive)
   - Bail with "transition X not available for N issues; available: ..." if not found in any group
   - Warn if some groups lack the transition (those issues are skipped)
8. Build (issue_keys, transition_id) pairs per workflow group
9. client.submit_bulk_transition(pairs, send_notification) → BulkTaskSubmitted
10. poll_until_done(task_id) → BulkTaskResult
11. print_bulk_result(result, global)
```

#### Bulk delete handler

```
1. jql branch detected
2. Require --yes (bail if missing)
3. Search issue keys: client.search_issue_keys(jql, max_results=1000)
4. Bail if 0 results
5. --dry-run: print intent and return
6. client.submit_bulk_delete(keys, send_notification) → BulkTaskSubmitted
7. poll_until_done(task_id) → BulkTaskResult
8. print_bulk_result(result, global)
```

#### `poll_until_done` helper

A free function in `issue.rs` (not a method):

```rust
async fn poll_until_done(
    client: &JiraClient,
    task_id: &str,
    global: &GlobalArgs,
) -> anyhow::Result<BulkTaskResult>
```

- Polls every 2 seconds via `tokio::time::sleep`
- In table mode: writes progress to stderr (not stdout) so it doesn't corrupt `-o json` output
- Stops when `status.is_terminal()`
- Returns `Err` if status is `Failed`, `Cancelled`, or `Dead`

#### Bulk create handler

```
1. from_file branch detected
2. Read and parse JSON file → Vec<serde_json::Value>
3. Map each value into JiraIssueCreate (reuse existing parsing helpers)
4. Bail with parse errors if any
5. --dry-run: print "Would POST /rest/api/3/issue/bulk for N issues (M batches)"
6. client.bulk_create_issues(&issues) → BulkCreateResult
7. print_bulk_create_result(result, global)
```

---

### Layer 5 — Format helpers (`atla-cli/src/commands/jira/format.rs`)

Add:

```rust
pub fn print_bulk_result(result: &BulkTaskResult, global: &GlobalArgs) -> anyhow::Result<()>
pub fn print_bulk_create_result(result: &BulkCreateResult, global: &GlobalArgs) -> anyhow::Result<()>
```

`print_bulk_result` renders:
- **table**: summary line + failure list
- **json**: `BulkTaskResult` serialised (snake_case keys)
- **keys**: one line per successfully processed key (looked up from `processedAccessibleIssues` IDs — requires a reverse ID→key mapping built during the key search step)
- **csv**: `key,status,error` rows

Note on keys output: the Jira bulk queue response returns issue **IDs** (integers) in
`processedAccessibleIssues`, not keys. To support `-o keys`, store a `id→key` map during
the initial JQL search step (the search response includes both `id` and `key`), and use it
to resolve IDs back to keys when printing.

---

## Chunking for large result sets

The bulk transition and delete APIs accept ≤1000 issues per request. The JQL search
side is **no longer** a concern: `JiraClient::search_issues` paginates
`/rest/api/3/search/jql` internally via `next_page_token`, so building
`search_issue_keys` on top of the same generated builder will inherit the same
behaviour for free. The remaining constraint is the **bulk submit batch size of 1000**.

When a JQL query matches more than 1000 issues:

1. Use `search_issue_keys(jql, max_results)` — paginates the search side automatically.
2. Split the resulting key list into chunks of 1000.
3. For transition: call `get_bulk_transitions` per chunk, submit one bulk task per chunk.
4. For delete: submit one bulk task per chunk.
5. Poll all `taskId`s concurrently with `tokio::try_join!` (or `futures::future::join_all`).
6. Aggregate `BulkTaskResult` totals before printing.

For the initial implementation, cap at 1000 issues and **emit a warning** (to stderr) if
the JQL matches more — for both `transition` and `delete`. Multi-chunk submit/poll can
be added in a follow-up; the JQL search side itself no longer needs follow-up work.

---

## Partial transition failures

The GET bulk transitions endpoint may return `isTransitionsFiltered: true` for some
workflow groups, meaning not all transitions are available due to field update
requirements. In that case:

- Print a **warning** listing the affected issues and skipping them from the bulk request.
- Continue with the remaining issues.
- Include the skipped count in the final summary.

If **no** workflow group has the requested transition, `bail!` with a clear error showing
which transitions are available.

---

## Adding bulk endpoints to `jira-v3-partial.json` (codegen path)

**Decision:** all bulk endpoints used by this feature go through the partial spec codegen
pipeline (progenitor). They are added to `jira-v3-partial.json` so that `atla-jira-api`
exposes typed builder methods and request/response structs for them. None of these
endpoints are called through `raw_client`.

### Endpoints and generated method names

The `operationId` from `specs/jira-v3.json` determines the method name generated by
progenitor (camelCase → snake_case):

| Endpoint | Method | `operationId` | Generated method |
|----------|--------|---------------|-----------------|
| `/rest/api/3/bulk/issues/transition` | GET | `getAvailableTransitions` | `self.generated.get_available_transitions()` |
| `/rest/api/3/bulk/issues/transition` | POST | `submitBulkTransition` | `self.generated.submit_bulk_transition()` |
| `/rest/api/3/bulk/issues/delete` | POST | `submitBulkDelete` | `self.generated.submit_bulk_delete()` |
| `/rest/api/3/bulk/queue/{taskId}` | GET | `getBulkOperationProgress` | `self.generated.get_bulk_operation_progress()` |
| `/rest/api/3/issue/bulk` | POST | `createIssues` | `self.generated.create_issues()` |

### Schema types to add to `simplifiedSchemas()`

Add the following entries to the `simplifiedSchemas()` function in
`scripts/jira-v3-partial-spec.js`, following the same hand-written pattern as existing
entries (e.g. `IssueUpdateDetails`, `CreatedIssue`). `IssueUpdateDetails` and `CreatedIssue`
are already present and can be referenced with `$ref`.

| Schema name | Used by |
|-------------|---------|
| `BulkTransitionGetAvailableTransitions` | Response for `GET /rest/api/3/bulk/issues/transition` |
| `IssueBulkTransitionPayload` | Request body for `POST /rest/api/3/bulk/issues/transition` |
| `SubmittedBulkOperation` | Response for `POST /rest/api/3/bulk/issues/transition` and `POST /rest/api/3/bulk/issues/delete` |
| `IssueBulkDeletePayload` | Request body for `POST /rest/api/3/bulk/issues/delete` |
| `BulkOperationProgress` | Response for `GET /rest/api/3/bulk/queue/{taskId}` |
| `IssuesUpdateBean` | Request body for `POST /rest/api/3/issue/bulk` |
| `CreatedIssues` | Response for `POST /rest/api/3/issue/bulk` |

Key fields to include (sourced from `specs/jira-v3.json`):

- `SubmittedBulkOperation`: `taskId: string`
- `BulkOperationProgress`: `taskId: string`, `status: string`, `progressPercent: integer`,
  `totalIssueCount: integer`, `processedAccessibleIssues: array (string items)`,
  `failedAccessibleIssues: object (additionalProperties: true)`,
  `invalidOrInaccessibleIssueCount: integer`
- `IssueBulkTransitionPayload`: `bulkTransitionInputs: array`, `sendBulkNotification: boolean`
- `IssueBulkDeletePayload`: `selectedIssueIdsOrKeys: array (string items)`,
  `sendBulkNotification: boolean`
- `BulkTransitionGetAvailableTransitions`: `availableTransitions: array`,
  `startingAfter: string`, `endingBefore: string`
- `IssuesUpdateBean`: `issueUpdates: array` with `items: { $ref: "#/components/schemas/IssueUpdateDetails" }`
- `CreatedIssues`: `issues: array` with `items: { $ref: "#/components/schemas/CreatedIssue" }`,
  `errors: array`

### Changes needed in `scripts/jira-v3-partial-spec.js`

1. **`simplifiedSchemas()`** — add the seven schema entries listed above.
2. **`selectedOperations`** — add five new path/method entries following the existing
   pattern. For example:

   ```js
   "/rest/api/3/bulk/issues/transition": {
     get: {
       parameters: [
         queryParameter("issueIdsOrKeys", { type: "string" }),
       ],
       response: "BulkTransitionGetAvailableTransitions",
     },
     post: {
       request: "IssueBulkTransitionPayload",
       responses: {
         201: jsonResponse("SubmittedBulkOperation"),
       },
     },
   },
   "/rest/api/3/bulk/issues/delete": {
     post: {
       request: "IssueBulkDeletePayload",
       responses: {
         201: jsonResponse("SubmittedBulkOperation"),
       },
     },
   },
   "/rest/api/3/bulk/queue/{taskId}": {
     get: {
       parameters: [
         pathParameter("taskId", { type: "string" }),
       ],
       response: "BulkOperationProgress",
     },
   },
   "/rest/api/3/issue/bulk": {
     post: {
       request: "IssuesUpdateBean",
       responses: {
         201: jsonResponse("CreatedIssues"),
       },
     },
   },
   ```

### Regeneration steps

After updating `scripts/jira-v3-partial-spec.js`:

1. Run `scripts/update-specs.sh`. This regenerates `specs/jira-v3-partial.json` and
   updates the sha256 checksums in `specs/manifest.json`.
2. Verify codegen compiles cleanly: `cargo build -p atla-jira-api`.
3. Commit `scripts/jira-v3-partial-spec.js`, `specs/jira-v3-partial.json`, and
   `specs/manifest.json` together.

---

## Acceptance criteria

- [ ] `atla jira issue transition --jql "..." --to X` transitions all matching issues
- [ ] `atla jira issue transition PROJ-123 --to X` still works unchanged
- [ ] `atla jira issue transition --jql "..."` without `--to` gives a clear error
- [ ] `atla jira issue delete --jql "..." --yes` deletes all matching issues
- [ ] `atla jira issue delete --jql "..."` without `--yes` gives a clear error
- [ ] `atla jira issue create --from-file tasks.json` creates all issues in the file
- [ ] `--dry-run` prints the intent without making any API calls
- [ ] `-o json` output is machine-readable and does not contain progress noise
- [ ] `-o keys` outputs only successfully processed keys, one per line
- [ ] `--no-notify` suppresses Jira bulk-change email notifications
- [ ] Transition with issues from multiple workflows resolves the correct `transitionId` per group
- [ ] If a transition is unavailable for some workflow groups, those issues are skipped with a warning
- [ ] Progress is printed to stderr (not stdout) in table mode
- [ ] `scripts/jira-v3-partial-spec.js` updated with the new endpoints and schemas, and `cargo build -p atla-jira-api` runs clean (no progenitor errors)
- [ ] `specs/jira-v3-partial.json` regenerated and `specs/manifest.json` checksums updated, both committed alongside the script change
