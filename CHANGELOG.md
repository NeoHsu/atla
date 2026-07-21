# Changelog

All notable changes to atla are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added scoped-token profiles with separate Jira and Confluence gateway roots via
  `auth login --cloud-id`, plus unauthenticated `auth discover --site` cloud-ID discovery.
- Added `auth login --token-stdin` for non-interactive secret input without process arguments.
- Added config schema v2 with automatic legacy backup/migration.
- Added a centralized operation registry, profile allow/deny policy, `--read-only`,
  `--max-pages`, `--max-items`, `--max-bytes`, and `--timeout` for agent-safe execution.
- Added additive JSON schema version 1, published schemas/fixtures, structured operation plans,
  expiring tamper-evident plan files, validated apply, and mutation receipts.
- Added exact Confluence page/blog JSON payload previews and binary attachment E2E coverage.
- Added explicit Confluence metadata views, top-level JSON `--fields` projection, Unicode-safe
  `--max-chars` body limits, self-describing body commands, and optional `spaceOwnerId` output.
- Added an exhaustive operation catalog check that keeps every CLI leaf, safety classification,
  destructive confirmation flag, HTTP method, and pagination marker synchronized.
- Added `cargo-deny` CI policy for dependency licenses, sources, advisories, wildcard versions, and
  duplicate-version visibility across every release target.
- Added a repository PR template that makes CLI, JSON, error, pagination, plan, mutation, security,
  documentation, and release-contract review explicit.
- Added an opt-in cross-worktree Cargo cache for fast CLI/core checks and a reproducible
  fresh-build benchmark.
- Expanded spec-refresh summaries to report normalized parameter, request, response, requiredness,
  enum, and nested schema contract changes in addition to operation/schema counts.
- Expanded the maintainer live-smoke ledger to every remote Jira, Confluence, and auth-discovery
  operation with selectable groups, mutation/resource budgets, cleanup, residue tracking, and
  API-drift versus CLI-regression failure classification.
- Added read-only `doctor`, `explain-policy`, `operation list`, and `schema list/print` discovery
  commands, including versioned JSON contracts and automatic schema-fixture coverage.
- Added a changelog, security/contributing policies, MSRV and macOS/Windows CI, a 53% line-coverage
  ratchet, scheduled spec-refresh PRs, Dependabot, and CycloneDX 1.5 SBOM generation.

### Security

- Updated transitive HTTP/3 dependencies to fix `RUSTSEC-2026-0185`.
- Updated `anyhow` to fix `RUSTSEC-2026-0190`.
- Added RustSec auditing and automated dependency updates to CI maintenance.
- Added request/connect/transfer deadlines and same-origin Basic-auth protection.
- Unified raw and generated-client retry under bounded, method-aware backoff with `Retry-After`;
  uncertain mutation outcomes remain non-retryable and classified as `ambiguous_mutation`.
- Made config, file-credential, and saved-plan writes atomic, synced, and mode `0600` on Unix.
- Pinned release actions and installer bytes, reduced workflow permissions, removed shell-template
  injection paths, and added local/global build-provenance attestations plus SBOM checksums.
- Added a repository gitleaks policy that excludes ignored Cargo artifacts and only allowlists
  Atlassian's literal `admin:admin` documentation example in the pinned upstream specification.

### Changed

- Refactored the bundled agent skill around fail-closed, atla-native mutation gates.
- Split the clap command model into domain modules under `crates/atla-cli/src/cli/`.
- Split Markdown-to-ADF parsing from ADF-to-Markdown rendering and added bidirectional golden
  fixtures for the conversion contract.
- Bounded `--all` requests now stop safely and emit a resume token.
- JSON body views and supplementary Jira/Confluence data remain a single JSON document or CSV
  schema; successful mutation objects include receipt metadata.
- Likely Markdown sent with the default storage representation now emits an actionable warning
  without silently changing the payload.
- Partial-spec filters now fail with contextual read/write errors instead of uncaught filesystem
  exceptions.
- CI and spec-refresh workflows now use the SHA-pinned Node 24 checkout action.
- Removed the obsolete unbudgeted ADF live-mutation helper; its cases remain covered by local
  Markdown/ADF tests and the bounded sandbox workflow.

### Fixed

- Converted Markdown bodies for Confluence blog create/update/comments instead of rejecting the
  documented representation.
- Omitted page/blog IDs from footer-comment reply payloads as required by Confluence Cloud.
- Kept attachment-upload and page-delete JSON output in versioned objects, and omitted false
  delete query flags instead of serializing them explicitly.
- Required comment bodies and Confluence space inputs during argument parsing so invalid commands
  fail before credentials, network access, or attachment uploads.
- Rejected saved plans larger than 1 MiB before writing them, kept unauthenticated `auth status`
  JSON machine-readable, and allowed exactly eight chained alias expansions as documented.
- Normalized upstream Confluence OpenAPI regressions so scheduled spec refreshes remain buildable,
  while keeping unsupported generated multipart upload operations excluded.
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

[Unreleased]: https://github.com/NeoHsu/atla/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/NeoHsu/atla/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/NeoHsu/atla/compare/v0.3.0...v0.5.0
[0.3.0]: https://github.com/NeoHsu/atla/compare/v0.2.3...v0.3.0
