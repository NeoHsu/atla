---
title: Saved Plans and Apply
description: Generate, inspect, validate, and execute tamper-evident mutation plans.
---

# Saved plans and apply

Saved plans separate local request construction from remote execution. Generation performs all
local parsing and Markdown→ADF conversion, writes a mode-`0600` JSON file atomically, and sends no
request.

```bash
atla plan jira issue create --project PROJ --type Task --summary 'Prepare launch' \
  --out create-plan.json
jq . create-plan.json
atla apply create-plan.json --yes --output json
```

`--out` and `--expires-in` can appear after the nested command. Plans expire after 3,600 seconds by
default; the allowed range is 1–86,400 seconds.

## Supported operations

| Operation ID | Plan command |
| --- | --- |
| `jira.issue.create` | `atla plan jira issue create ... --out FILE` |
| `confluence.page.create` | `atla plan confluence page create --space-id 123 ... --out FILE` |
| `confluence.page.update` | `atla plan confluence page update 456 ... --out FILE` |
| `confluence.blog.create` | `atla plan confluence blog create --space-id 123 ... --out FILE` |
| `confluence.blog.update` | `atla plan confluence blog update 789 ... --out FILE` |

Confluence create plans require a numeric `--space-id`, avoiding a network lookup. Full page and
blog updates require explicit title/body/version values when the current remote value would
otherwise need to be loaded. Mention search is deliberately unavailable; pass explicit
`--mention NAME=ACCOUNT_ID` mappings.

## Plan contents

A plan records schema/plan versions, canonical operation ID, profile/site, exact HTTP request,
preconditions, unresolved values, input-file SHA-256 values, creation/expiration timestamps, and a
SHA-256 plan digest. It never records an API token or Authorization header.

Input paths are canonical absolute paths. If an input file changes before apply, execution fails.
The plan output cannot overwrite one of its own input files.

## Apply safety gates

`atla apply FILE --yes` rejects the plan before network access unless all checks pass:

1. file size, JSON shape, `schemaVersion`, and `planVersion`;
2. plan SHA-256 digest and validity window (maximum 24 hours);
3. active profile and site match;
4. profile allow/deny policy permits the original operation ID;
5. unresolved values are empty and every input-file hash still matches;
6. operation, method, API path, query parameters, and same-origin URL are on the built-in allowlist;
7. exactly one JSON request is present;
8. explicit `--yes` confirmation is present.

Apply is not arbitrary HTTP replay. Editing the URL/body or changing the operation invalidates the
digest, and recomputing a digest still cannot bypass the profile, policy, origin, route, method, or
query allowlists.

The plan hash is tamper-evident, **not a cryptographic signature**. Anyone who can edit a plan can
compute a new digest, so review plans and accept them only from a trusted source. Global `--read-only` blocks both apply and writing a saved plan; use stdout-only
`--dry-run --output json` when operating in that mode.

## Receipts and uncertain outcomes

Successful apply writes one JSON mutation result with receipt metadata: operation, profile, target,
request ID when available, and completion time. Non-idempotent timeout/server failures use
`ambiguous_mutation`; inspect the remote target before deciding whether to create and apply a new
plan.
