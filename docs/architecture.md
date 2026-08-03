---
title: atla Architecture
description: Module boundaries, source-of-truth contracts, and quality-gate layers for atla.
---

# atla architecture

`atla` is a machine-facing Jira and Confluence CLI. Its boundaries are designed around stable
contracts: the CLI owns invocation and output policy, `atla-core` owns domain behavior, and the
three API crates provide generated transport models behind a hand-written safety boundary.

## System overview

```text
+--------------------------+
| atla CLI                 |
| clap, policy, output,    |
| pagination, plan checks  |
+------------+-------------+
             |
             v
+--------------------------+
| atla-core                |
| auth, profiles, Jira,    |
| Confluence, Markdown/ADF |
+------------+-------------+
             |
             v
+--------------------------+
| GeneratedTransport       |
| retries, backoff, body   |
| reading, mutation safety |
+------------+-------------+
             |
      +------+------+
      |             |
      v             v
+-----------+  +----------------+
| Jira API  |  | Confluence API |
| generated |  | generated      |
+-----------+  +----------------+
```

Generated Progenitor code is built into `OUT_DIR` and is not a source boundary for CLI code. The
CLI consumes hand-written models and behavior from `atla-core`; generated builders execute only
through `GeneratedTransport`.

## Workspace map

| Path | Responsibility | Boundary |
| --- | --- | --- |
| `crates/atla-cli/src/cli/` | Clap definitions and global flags | Public command surface |
| `crates/atla-cli/src/commands/` | Command orchestration and local IO | Dispatches to core/domain behavior |
| `crates/atla-cli/src/operation/` | Operation IDs, risk metadata, and plan routes | Single safety-policy catalog |
| `crates/atla-cli/src/output/` | Table, JSON, CSV, key, schema, and error rendering | Stable machine-readable output |
| `crates/atla-core/src/jira/` | Jira domain clients and models | No CLI formatting or command matching |
| `crates/atla-core/src/confluence/` | Confluence domain clients and models | No CLI formatting or command matching |
| `crates/atla-core/src/profile/` | Profile config, policy, storage, and migration modules | Atomic local configuration boundary |
| `crates/atla-core/src/markdown/` | Markdown/ADF conversion | Shared content boundary |
| `crates/atla-cli/src/commands/{jira,confluence}/format/` | Domain-specific rendering, parsing, and body helpers | CLI output and input boundary |
| `crates/atla-core/src/generated_api.rs` | Generated-client ownership and transport | Keeps generated APIs behind policy |
| `crates/*-api/build.rs` | Partial-spec Progenitor generation | Build-time only; output is not committed |
| `specs/` | Upstream and filtered API contracts | Reproducible code-generation input |
| `docs/schemas/` | Agent-facing JSON schemas and fixtures | Checked contract mirror |
| `skills/atla-cli/` | Agent-facing command guidance | Must remain in CLI/version lockstep |

## Source of truth

| Contract | Source of truth | Verification |
| --- | --- | --- |
| CLI commands and flags | Clap definitions under `crates/atla-cli/src/cli/` | `cli_surface`, `doc_examples_parse`, docs tests |
| Mutation safety | `operation/registry.rs` and `operation.rs` | Operation catalog tests and E2E policy tests |
| JSON output | Renderer/schema code and `docs/schemas/` | Schema fixtures and output contract tests |
| API clients | `specs/*-partial.json` plus build filters | Spec filter reproduction and manifest hash tests |
| Agent skill | `skills/atla-cli/` | Exact CLI/skill/version lockstep check |
| Release artifacts | cargo-dist output plus hardening scripts | Checksums, SBOM, archive verifier, attestations |

The same behavior must not be reimplemented independently in a document, skill, schema, or
fixture. When a public contract changes, update its source of truth first and then regenerate or
validate the checked mirrors.

## Test organization

Integration tests are grouped by contract responsibility under `crates/atla-cli/tests/e2e/`:

| Module | Focus |
| --- | --- |
| `support.rs` | Process invocation, temporary config, and WireMock helpers |
| `output.rs` and `formats.rs` | Exit codes, errors, and output representations |
| `auth.rs` and `policy.rs` | Profiles, discovery, doctor, read-only, and mutation guards |
| `plans.rs` | Saved plans, dry-run previews, and mutation receipts |
| `confluence.rs` | Page/blog/attachment/body conversion and projection behavior |
| `pagination.rs` | Page, item, byte, and resume-token budgets |
| `transport.rs` | Retry, timeout, generated-client, and attachment transport policy |

This keeps the integration-test root as a module index and makes responsibility-specific failures
quick to locate without weakening the full `cargo nextest` gate. Core tests remain close to domain
modules, while `scripts/tests/` verifies repository tooling and spec-generation invariants.

## Quality-gate layers

| Layer | Entry point | Contract enforced |
| --- | --- | --- |
| Inner loop | `mise run check:fast` | Fast CLI compilation with shared target cache |
| Focused checks | `mise run lint`, `mise run test:*`, `mise run contract:check` | One subsystem at a time |
| Security | `mise run security` and `mise run workflow:security` | Secrets, dependency policy, advisories, workflow security |
| Pull request | `mise run check:pr` | Locked dependencies, lint, tests, MSRV, contracts, security, workflows |
| CI | `.github/workflows/ci.yml` | Parallel Rust quality, coverage, MSRV, platform, and workflow gates |
| Release | `.github/workflows/release.yml` | Repeated release checks, checksums, SBOM, provenance, and artifact verification |

CI jobs use explicit timeouts and cancel obsolete runs on the same ref. Cargo commands that affect
resolution use `--locked`, so the checked-in lockfile is part of the validation contract rather
than an incidental local artifact.

## Dependency direction

```text
CLI definitions / commands / output
                |
                v
             atla-core
                |
                v
 GeneratedTransport and hand-written models
                |
                v
 Generated Jira / Confluence API crates
```

The reverse dependency is forbidden: core must not depend on CLI output or command policy, and
CLI code must not reach into generated API clients directly. New remote operations should be
represented as domain behavior plus an explicit operation-catalog entry, not as an ad-hoc HTTP
replay.
