---
title: Feature Matrix
description: Complete Jira, Confluence, output, safety, and integration capability overview.
---

# Feature matrix

This page is the high-level inventory of the supported `atla` surface. For exact flags and
examples, use the [Jira](jira.md), [Confluence](confluence.md), or
[Agent Reference](agent-reference.md) documentation.

## Core

| Area | Capabilities |
| --- | --- |
| Authentication | `login`, `discover`, `logout`, `status`, and `switch` with named profiles |
| Configuration | `set`, `get`, and `list`; aliases, environment overrides, and schema migration |
| Discovery | `doctor`, `explain-policy`, `operation list`, `schema list`, and `schema print` |
| Saved plans | `plan jira`, `plan confluence`, and policy-checked `apply` |
| Shell integration | Completion generation for bash, zsh, fish, and PowerShell |
| Agent integration | Version-matched `atla-cli` skill with compatibility checks and command references |

## Jira

| Resource | Capabilities |
| --- | --- |
| Projects | `list`, `view`, and `issue-types` |
| Search | JQL search with table, JSON, CSV, and key output |
| Issues | `list`, `create`, `view`, `update`, `edit`, and guarded `delete` |
| Issue fields | Create metadata with required flags, types, and allowed values |
| Assignment | Assign to `me`, an account ID, or a user query |
| Transitions | List or apply transitions, with interactive selection when safe |
| Comments | `add`, `list`, `update`, and guarded `delete` |
| Attachments | `list`, `upload`, `download`, and guarded `delete` |
| Links | `add`, `list`, `remove`, `github-links`, and `github-commits` |
| Worklogs | `add` and `list` |
| Boards | `list` with project/type/name filters and `view` |
| Sprints | `list`, `active`, `view`, `create`, `start`, `close`, `add`, `remove`, and `issues` |

## Confluence

| Resource | Capabilities |
| --- | --- |
| Spaces | `list`, `view`, `create`, `update`, and guarded `delete` |
| Pages | `list`, `view`, `create`, `update`, `move`, guarded `delete`, `children`, and `copy` |
| Page content | Storage, wiki, ADF, and Markdown input; storage, ADF, and Markdown output |
| Page labels | `list`, `add`, and `remove` |
| Page comments | `add`, `list`, and guarded `delete` |
| Blogs | `list`, `view`, `create`, `update`, and guarded `delete` |
| Blog labels | `list`, `add`, and `remove` |
| Blog comments | `add`, `list`, and guarded `delete` |
| Search | CQL search through the scoped Confluence v1 REST endpoint |
| Attachments | `list`, `view`, `upload`, `download`, and guarded `delete` |

## Output and safety

| Area | Capabilities |
| --- | --- |
| Output formats | Human-readable tables plus stable `json`, `csv`, and `keys` output |
| JSON contracts | Additive schema version 1 objects, fixtures, and published JSON Schemas |
| Pagination | Opaque query-bound page tokens, ready-to-run next commands, and resumable `--all` |
| Mutation policy | Global `--dry-run`, `--read-only`, explicit `--yes`, and profile allow/deny rules |
| Context budgets | `--max-pages`, `--max-items`, `--max-bytes`, and `--timeout` |
| Receipts | Operation, profile, target, request ID when available, and completion time |
| Supply chain | Checksummed archives/installers, provenance attestations, and CycloneDX 1.5 SBOM |
