---
title: Operation Policy and Context Budgets
description: Read-only enforcement, profile allow/deny rules, operation IDs, and agent budgets.
---

# Operation policy and context budgets

atla classifies every parsed command in a central operation registry before loading credentials
or sending a request. Each operation has a stable ID, HTTP method, pagination marker, and effect:
read, write, or destructive. `--verbose` prints this metadata to stderr.

## Global read-only mode

Use `--read-only` or `ATLA_READ_ONLY=1` for discovery and planning agents:

```bash
atla --read-only --output json jira issue view PROJ-123
```

A real local or remote mutation fails with exit code 2 before credential or network access.
`--yes` cannot bypass the policy. A `--dry-run` mutation is allowed because it only prints a
preview:

```bash
atla --read-only --dry-run jira issue delete PROJ-123 --yes
```

Read-only config loading does not rewrite or migrate the config file. `atla plan --out FILE` is
blocked under global read-only because it writes a local artifact; use `--dry-run --output json`
for a stdout-only plan.

## Profile policy

Profiles can define a default mode plus operation allow/deny patterns:

```toml
[profiles.agent.policy]
mode = "read-only"
allow = ["jira.issue.comment.add", "confluence.page.update"]
deny = ["*.delete", "jira.issue.transition"]
```

The same values can be managed from the CLI:

```bash
atla --profile agent config set policy-mode read-only
atla --profile agent config set policy-allow jira.issue.comment.add,confluence.page.update
atla --profile agent config set policy-deny '*.delete,jira.issue.transition'
```

Evaluation order is deterministic:

1. matching `deny` blocks;
2. matching `allow` permits;
3. `mode` supplies the default (`read-only` permits reads; `read-write` permits all effects);
4. destructive commands still require `--yes`.

Patterns are anchored to the complete operation ID. `*` matches any sequence, including dots;
there are no implicit prefixes or regular expressions. For example, `*.delete` matches
`jira.issue.delete` and `confluence.page.delete`, while `jira.issue.*` matches only IDs under
that prefix.

Profile policy governs product operations. Local auth/config management and read-only discovery
commands remain available so a user can recover from an overly restrictive profile. Global
`--read-only` still blocks local writes. Dry-run previews are permitted without weakening execution
policy.

Inspect the stable operation catalog and explain a concrete decision without credentials or
network access:

```bash
atla operation list --output json
atla --profile agent explain-policy jira.issue.create --output json
atla --profile agent --read-only explain-policy jira.issue.create --output json
```

The explanation reports the matching deny or allow pattern, the profile mode fallback, and whether
global `--read-only` independently blocks the operation. Deny remains higher priority than allow.

## Context budgets

Bound broad agent reads with global limits:

```bash
atla --read-only --max-pages 5 --max-items 200 --max-bytes 1000000 --timeout 30 \
  --output json confluence search 'type = page AND label = runbook'
```

| Flag | Enforcement |
| --- | --- |
| `--max-pages N` | Stops automatic pagination after N API pages across the operation |
| `--max-items N` | Caps records collected by list/search clients |
| `--max-bytes N` | With `--output json`, refuses oversized output before printing it |
| `--timeout N` | Applies an N-second deadline to API requests, uploads, and downloads |

When an item/page budget stops `--all`, atla emits the normal opaque resume token. A byte-budget
violation uses exit code 2 and does not print a partial structured document.

## Mutation uncertainty

POST/PATCH-style mutations are not automatically retried after timeouts or server errors. If the
remote side may have committed the operation, JSON errors use `kind=ambiguous_mutation`, exit code
1, and `retryable=false`. Query the target state before deciding whether to repeat the operation.
