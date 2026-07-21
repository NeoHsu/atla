---
title: Compatibility Policy
description: Supported Rust, platforms, Atlassian products, config, and JSON contracts.
---

# Compatibility policy

| Surface | v1 support target |
| --- | --- |
| Atlassian deployment | Jira Cloud and Confluence Cloud |
| Unsupported deployments | Jira/Confluence Server and Data Center |
| Rust MSRV | 1.91 |
| Config | schema v2; automatic migration from unversioned v1 |
| JSON objects | schemaVersion 1; additive changes only within the version |
| Operation plans | planVersion 1 for documented commands |
| Agent skill | Exact release lockstep: skill version must equal the `atla` CLI version |
| Auth | unscoped/scoped API tokens; OAuth 3LO deferred |
| Release platforms | macOS arm64/x86_64, Linux arm64/x86_64, Windows x86_64 |

## Stability rules

Before 1.0, a release may introduce a documented compatibility change, but it must include
migration notes and a changelog entry. At 1.0, command/flag names, exit-code meanings, JSON v1
fields, pagination token behavior, destructive `--yes`, dry-run no-write behavior, and policy
precedence are frozen for the v1 line.

Patch and minor releases may:

- add commands, flags, optional JSON fields, or enum values;
- fix behavior that violated the documented contract;
- update generated clients and dependencies without changing the public contract.

They may not silently remove or rename a command/field, reinterpret an opaque pagination token for
a different query, or make a previously non-interactive agent command prompt.

## Agent skill version lockstep

Every released `skills/atla-cli` package targets exactly one CLI version. The machine-readable
`skills/atla-cli/compatibility.json`, both Cargo packages, `Cargo.lock`, release tag, SKILL.md gate,
and tag-pinned installation docs must carry the same SemVer. CI verifies this contract.

Before any auth, credential, network, or Atlassian operation, the skill runs:

```bash
atla doctor --skill-version 0.6.0 --output json
```

A mismatch exits with code `2` and `kind=version_mismatch`, returns a tagged `updateCommand`, and
performs no additional checks. The agent shows that command and waits for user approval; neither
the skill nor CLI updates local agent configuration automatically. Published installation commands
use `https://github.com/NeoHsu/atla/tree/v0.6.0`, never an unversioned default branch.

See [JSON Contracts](json-schemas.md), [Config Migration](migration-v1.md), and
[Authentication ADR](adr/0001-auth-and-endpoint-model.md).
