---
title: JSON Contracts
description: Versioned JSON envelopes, plans, receipts, and compatibility policy.
---

# JSON contracts

Machine-readable object output includes `"schemaVersion": 1`. Version 1 is additive: patch and
minor releases may add fields, but do not rename/remove fields or change their type. Consumers
should ignore unknown fields. A future breaking shape requires a new schema version and migration
notes.

Published schemas:

- [`schemas/error-v1.schema.json`](schemas/error-v1.schema.json)
- [`schemas/list-v1.schema.json`](schemas/list-v1.schema.json)
- [`schemas/jira-issue-list-v1.schema.json`](schemas/jira-issue-list-v1.schema.json)
- [`schemas/confluence-page-list-v1.schema.json`](schemas/confluence-page-list-v1.schema.json)
- [`schemas/operation-plan-v1.schema.json`](schemas/operation-plan-v1.schema.json)
- [`schemas/mutation-receipt-v1.schema.json`](schemas/mutation-receipt-v1.schema.json)
- [`schemas/doctor-v1.schema.json`](schemas/doctor-v1.schema.json)
- [`schemas/operation-list-v1.schema.json`](schemas/operation-list-v1.schema.json)
- [`schemas/policy-explanation-v1.schema.json`](schemas/policy-explanation-v1.schema.json)
- [`schemas/schema-list-v1.schema.json`](schemas/schema-list-v1.schema.json)

Every schema has a representative file under [`schemas/fixtures/`](schemas/fixtures/). Tests
automatically discover every `*.schema.json`, require its matching fixture, and check required,
type, enum, const, property, and array constraints.

Discover these contracts from an installed binary without locating repository files:

```bash
atla schema list --output json
atla schema print operation-list-v1 --output json
```

`schema print` emits the bundled schema itself and does not inject an output `schemaVersion` field
into the schema document.

`doctor-v1` includes the running `cliVersion` and optional `skillCompatibility`. With
`--skill-version`, the nested object reports exact compatibility, the target version,
`recommendedAction`, and a nullable `updateCommand`. A mismatch still emits this report on stdout,
then exits `2` with a `version_mismatch` error object on stderr; no config, credential, or network
check runs.

## Lists

List/search objects retain their command-specific item key (`issues`, `results`, `projects`, and
so on) and the stable pagination object:

```json
{
  "schemaVersion": 1,
  "issues": [],
  "pagination": {
    "isLast": true,
    "nextPageToken": null,
    "nextCommand": null
  }
}
```

`nextPageToken` is opaque and query-bound. If a global item/page budget truncates `--all`,
`isLast` is false and a resume token is present.

## Errors

JSON runtime errors are written to stderr. `retryable` and process exit code are the control
signals; `message` is explanatory text and is not a stable parser key.

## Operation plans

Selected JSON-body mutations support a deterministic plan with `--output json --dry-run` or a
saved, expiring plan with `atla plan ... --out FILE`. `planVersion` versions plan semantics
independently of ordinary result schemas. Plans contain no token and make no request. Their URLs and
bodies are the values that the real command would use after local conversion.

Saved plans add creation/expiration times, optional input-file hashes, and a SHA-256 digest over the
canonical plan. `atla apply FILE --yes` rejects changed, expired, profile/site-mismatched,
policy-blocked, unresolved, multi-request, cross-origin, or non-allowlisted plans before loading the
token or sending a request. The digest is tamper-evident, not a signature; only apply plans from a
trusted source. See [Saved Plans and Apply](./plans.md) for commands and validation gates.

Jira issue create and Confluence page/blog create/update currently support saved plans. Other
dry-run commands retain their human-readable preview without `--output json`; requesting JSON for
an unsupported dry-run fails with exit code 2 and leaves stdout empty rather than mixing prose with
JSON.

## Mutation receipts

Successful mutation JSON objects include receipt metadata (`operation`, `profile`, `target`,
`requestId`, `completedAt`) while retaining command result fields at the top level. `target` is
null when the command result has no unambiguous primary ID. `requestId` is null when Atlassian does
not expose one through the client response.
