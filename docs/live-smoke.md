---
title: Live Sandbox Smoke Testing
description: Bounded Jira and Confluence contract testing with auditable coverage and cleanup.
---

# Live sandbox smoke testing

Mock E2E tests remain the default. Maintainers use `scripts/atla-live-smoke.py`
only against a dedicated Atlassian sandbox when live API behavior or a release
candidate needs validation. The ledger does not send network requests; it
validates evidence produced by separately executed `atla` commands.

Never run this workflow against production. Keep `.atla_live/` evidence local;
it may contain tenant URLs, issue keys, page IDs, and response data.

## Coverage groups

The ledger mirrors every remote operation in the Rust operation catalog. A test
keeps the two registries synchronized.

| Group | Scope | Operations |
| --- | --- | ---: |
| `auth-discovery` | Cloud-ID and product endpoint discovery | 1 |
| `jira-issue-lifecycle` | Projects, boards, issues, comments, links, worklogs, and sprints | 32 |
| `jira-attachment-lifecycle` | Jira attachment list/upload/download/delete | 4 |
| `confluence-page-lifecycle` | Spaces, pages, blogs, labels, and comments | 30 |
| `confluence-attachment-lifecycle` | Confluence attachment list/view/upload/download/delete | 5 |
| `cql-jql-search` | Bounded JQL issue list/search and CQL search | 3 |

Pass `--group` repeatedly to select the scope. Unselected operations are
recorded as skipped, so `finish` cannot confuse a focused run with full
coverage.

## Safety invariants

- `--max-mutations` and `--max-resources` are mandatory positive budgets with
  conservative defaults of 50.
- Mutation results cannot pass unless the ledger was initialized with
  `--allow-mutations` and auth evidence reports `policy_mode: read-write`.
- The baseline page and issue are protected and can never be tracked as
  disposable resources.
- Updates and deletes must reference a resource already tracked by the ledger.
- Child resources such as comments and attachments require a tracked temporary
  parent.
- Space mutations require the separate `--allow-space-mutations` gate and a
  temporary space key.
- Non-reversible worklog/sprint mutations are skipped unless
  `--allow-residue` is explicit. Any remaining resource needs a recorded
  residue reason.
- `finish` fails on pending operations, failures, active resources, or trashed
  resources awaiting purge.

## 1. Capture preflight evidence

Use one sandbox profile and store evidence outside version control:

```bash
mkdir -p .atla_live
atla --profile sandbox --output json auth status > .atla_live/auth.json
atla --profile sandbox --read-only --output json confluence page view 123456 \
  > .atla_live/page-baseline.json
atla --profile sandbox --read-only --output json jira issue view SANDBOX-1 \
  > .atla_live/issue-baseline.json
```

The protected page and issue should be stable fixtures that the run must not
modify. Capture their final state again after cleanup and compare it with these
baselines.

## 2. Initialize a bounded ledger

This example selects all six groups but does not authorize mutations:

```bash
python3 scripts/atla-live-smoke.py init \
  --state .atla_live/state.json \
  --auth-status .atla_live/auth.json \
  --confluence-baseline .atla_live/page-baseline.json \
  --jira-baseline .atla_live/issue-baseline.json \
  --site https://example.atlassian.net \
  --profile sandbox \
  --target-page 123456 \
  --target-issue SANDBOX-1 \
  --space-key SBX \
  --project-key SANDBOX \
  --group auth-discovery \
  --group jira-issue-lifecycle \
  --group jira-attachment-lifecycle \
  --group confluence-page-lifecycle \
  --group confluence-attachment-lifecycle \
  --group cql-jql-search \
  --max-mutations 30 \
  --max-resources 20
```

For an approved mutation run, recreate the ledger with `--allow-mutations`.
Add `--allow-purge`, `--allow-space-mutations --temporary-space-key SBXTMP`, or
`--allow-residue` only when the sandbox owner has approved those exact effects.

## 3. Execute and classify operations

Save command output before recording a pass:

```bash
atla --profile sandbox --read-only --output json jira issue view SANDBOX-1 \
  > .atla_live/jira-issue-view.json
python3 scripts/atla-live-smoke.py record \
  --state .atla_live/state.json \
  --operation jira.issue.view \
  --result pass \
  --evidence .atla_live/jira-issue-view.json
```

A created resource must be registered from the mutation response:

```bash
atla --profile sandbox --no-input --output json jira issue create \
  --project SANDBOX \
  --type Task \
  --summary "atla live smoke 20260720" \
  > .atla_live/jira-issue-create.json
python3 scripts/atla-live-smoke.py record \
  --state .atla_live/state.json \
  --operation jira.issue.create \
  --result pass \
  --evidence .atla_live/jira-issue-create.json \
  --resource jira-issue:SANDBOX-2
```

Classify every failure so reports distinguish an Atlassian contract change from
an atla regression or a sandbox/network problem:

```bash
python3 scripts/atla-live-smoke.py record \
  --state .atla_live/state.json \
  --operation confluence.search \
  --result fail \
  --failure-class api-drift \
  --evidence .atla_live/confluence-search-error.txt
```

Use `--result skip --reason "..."` for an intentionally inapplicable operation.
A mutation with an ambiguous response must be reconciled by a read-back; if it
created a resource, register it with `resource-add` before continuing.

## 4. Clean up in reverse order

Generate commands only from tracked resource IDs:

```bash
python3 scripts/atla-live-smoke.py cleanup-commands \
  --state .atla_live/state.json
```

Execute each printed command after review. Then record the observed state:

```bash
python3 scripts/atla-live-smoke.py resource-set \
  --state .atla_live/state.json \
  --type jira-issue \
  --id SANDBOX-2 \
  --to deleted
```

Confluence pages, blogs, and attachments may transition through `trashed` and
then `purged` when purge was authorized. If Jira exposes no cleanup operation,
a run initialized with `--allow-residue` may record `--to residue --reason
"sandbox owner approved retained sprint 77"`. Residue is visible in the final
summary and is never treated as deletion.

## 5. Finish and archive the result

```bash
python3 scripts/atla-live-smoke.py status \
  --state .atla_live/state.json \
  --json
python3 scripts/atla-live-smoke.py finish \
  --state .atla_live/state.json
```

A release gate passes only when `finish` exits zero. Preserve a redacted summary
outside the repository when release evidence is required; do not commit the raw
ledger or response files.
