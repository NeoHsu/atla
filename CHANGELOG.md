# Changelog

All notable changes to atla are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added an exhaustive operation catalog check that keeps every CLI leaf, safety classification,
  destructive confirmation flag, HTTP method, and pagination marker synchronized.
- Added `cargo-deny` CI policy for dependency licenses, sources, advisories, wildcard versions, and
  duplicate-version visibility across every release target.
- Added a repository PR template that makes CLI, JSON, error, pagination, plan, mutation, security,
  documentation, and release-contract review explicit.
- Added an opt-in cross-worktree Cargo cache for fast CLI/core checks, a reproducible fresh-build
  benchmark, and generated operation/schema summaries for spec-refresh PRs.

### Changed

- Split the clap command model into domain modules under `crates/atla-cli/src/cli/` without
  changing the generated CLI surface.
- Split Markdown-to-ADF parsing from ADF-to-Markdown rendering and added bidirectional golden
  fixtures for the conversion contract.

### Fixed

- Normalized upstream Confluence OpenAPI regressions so scheduled spec refreshes remain buildable,
  while keeping unsupported generated multipart upload operations excluded.

## [0.6.0] - 2026-07-17

### Added

- Added scoped-token profiles with separate Jira and Confluence gateway roots via
  `auth login --cloud-id`, plus unauthenticated `auth discover --site` cloud-ID discovery.
- Added `auth login --token-stdin` for non-interactive secret input without process arguments.
- Added config schema v2 with automatic legacy backup/migration.
- Added a centralized operation registry, profile allow/deny policy, `--read-only`,
  `--max-pages`, `--max-items`, `--max-bytes`, and `--timeout` for agent-safe execution.
- Added exact Confluence page/blog JSON payload previews and binary attachment E2E coverage.
- Added additive JSON schema version 1, published schemas/fixtures, structured operation plans,
  expiring tamper-evident plan files, validated apply, and mutation receipts.
- Added a changelog, security/contributing policies, MSRV and macOS/Windows CI, a coverage
  baseline artifact, scheduled spec-refresh PRs, Dependabot, and CycloneDX 1.5 SBOM generation.

### Security

- Updated transitive HTTP/3 dependencies to fix `RUSTSEC-2026-0185`.
- Updated `anyhow` to fix `RUSTSEC-2026-0190`.
- Added RustSec auditing and automated dependency updates to CI maintenance.
- Added request/connect/transfer deadlines and same-origin Basic-auth protection.
- Made retries method-aware across raw and generated clients; uncertain mutation outcomes are
  non-retryable and classified as `ambiguous_mutation`.
- Made config, file-credential, and saved-plan writes atomic, synced, and mode `0600` on Unix.
- Pinned release actions and installer bytes, reduced workflow permissions, removed shell-template
  injection paths, and added local/global build-provenance attestations plus SBOM checksums.

### Changed

- Refactored the bundled agent skill around fail-closed, atla-native mutation gates.
- Added a tested maintainer-only Confluence live-smoke coverage/resource ledger to the repository.
- Bounded `--all` requests now stop safely and emit a resume token.
- Generated clients now retry transient read/idempotent failures under a bounded policy.
- JSON body views and supplementary Jira/Confluence data now remain a single JSON document or CSV
  schema; successful mutation objects include receipt metadata.

### Fixed

- Converted Markdown bodies for Confluence blog create/update/comments instead of rejecting the
  documented representation.
- Omitted page/blog IDs from footer-comment reply payloads as required by Confluence Cloud.
- Kept attachment-upload and page-delete JSON output in versioned objects, and omitted false
  delete query flags instead of serializing them explicitly.
- Corrected API-token expiration guidance and Confluence v2 code-generation documentation.
- Kept content `--version` flags in the generated CLI surface snapshot.
- Made spec manifest refresh timestamps update on every refresh.

## [0.5.1] - 2026-07-10

### Changed

- Replaced line-oriented Markdown parsing with a CommonMark AST powered by `comrak`.

## [0.5.0] - 2026-07-08

### Added

- Machine-readable runtime error output and classified exit codes for agents.
- End-to-end tests of errors, output formats, pagination, dry-run behavior, and retry handling.
- JSON request-body previews for Jira issue and comment mutations in dry-run mode.
- Complete clap help text and documentation drift checks for examples and the CLI surface.

### Changed

- Pruned the generated Confluence v2 client to used operations.
- Unified generated-client authentication and raw-request transient retry handling.

## [0.3.0] - 2026-06-18

### Added

- Markdown table metadata preservation and numbered-row options.
- Markdown mention conversion for Confluence.
- Attachment references in Jira and Confluence comments.
- Bundled agent-skill installation documentation.

### Fixed

- Accepted numeric Jira attachment identifiers.

[Unreleased]: https://github.com/NeoHsu/atla/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/NeoHsu/atla/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/NeoHsu/atla/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/NeoHsu/atla/compare/v0.3.0...v0.5.0
[0.3.0]: https://github.com/NeoHsu/atla/compare/v0.2.3...v0.3.0
