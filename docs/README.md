---
title: atla Documentation
description: User, automation, agent, and maintainer documentation for atla.
---

# atla documentation

Use this index when you need more detail than the project homepage. Current-contract command
examples in these documents are checked against the real clap definition during tests.

## Start here

| Guide | What it covers |
| --- | --- |
| [Getting Started](getting-started.md) | Installation, first-time setup, shell completions, and a quick demo |
| [Feature Matrix](features.md) | Complete Jira, Confluence, output, and safety capability overview |
| [Compatibility](compatibility.md) | Supported platforms and the v1 compatibility policy |

## Configure atla

| Guide | What it covers |
| --- | --- |
| [Authentication](authentication.md) | Auth commands, multiple profiles, token storage, and environment variables |
| [Configuration](configuration.md) | Config keys, aliases, config schema, and environment overrides |
| [Config Migration](migration-v1.md) | Schema v2 migration, backups, and rollback |

## Use Jira and Confluence

| Guide | What it covers |
| --- | --- |
| [Jira](jira.md) | Projects, issues, boards, sprints, comments, attachments, and worklogs |
| [Confluence](confluence.md) | Spaces, pages, blogs, labels, comments, attachments, and CQL search |
| [Output Formats](output-formats.md) | Global flags, table/JSON/CSV/key output, pagination, and context budgets |

## Automate safely

| Guide | What it covers |
| --- | --- |
| [JSON Contracts](json-schemas.md) | Versioned schemas, fixtures, receipts, and compatibility guarantees |
| [Saved Plans](plans.md) | Generate, inspect, validate, and apply expiring mutation plans |
| [Operation Policy](policy.md) | Read-only enforcement, profile allow/deny rules, and context budgets |
| [Agent Reference](agent-reference.md) | Structured command and error reference for agents and automation |

## Maintain and release

| Guide | What it covers |
| --- | --- |
| [Contributing](../CONTRIBUTING.md) | Development checks, CLI contracts, and the PR process |
| [Security Policy](../SECURITY.md) | Supported versions and vulnerability reporting |
| [Changelog](../CHANGELOG.md) | User-visible changes by release |
| [Code Generation](code-generation.md) | Progenitor clients, partial-spec filtering, and the refresh workflow |
| [Release Procedure](releasing.md) | Versioning, artifacts, checksums, SBOM, provenance, and workflow hardening |
| [Live Sandbox Smoke Testing](live-smoke.md) | Bounded Jira/Confluence validation and cleanup ledger |
| [Authentication and Endpoint ADR](adr/0001-auth-and-endpoint-model.md) | Auth, profile, and endpoint design decisions |
