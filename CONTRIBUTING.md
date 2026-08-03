# Contributing to atla

Thank you for improving atla. The CLI is consumed by automation and coding agents, so deterministic
behavior and machine-readable contracts are part of the public API.

## Development setup

Install Rust 1.91 or newer. The portable Rust baseline is:

```bash
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked \
  --exclude atla-jira-api --exclude atla-confluence-api --exclude atla-confluence-v1-api -- -D warnings
cargo audit
cargo deny check
```

These explicit commands remain the portable Rust baseline. CI runs the same workspace unit and
integration tests with `cargo-nextest`, then runs doctests separately so test coverage is not lost.
It also scans the source tree with gitleaks, checks direct Cargo dependency liveness with
`cargo-machete`, and validates the repository's Python maintenance scripts with Ruff. Contributors
using `mise` can run the common workflows through discoverable convenience tasks:

| Task | Purpose |
| --- | --- |
| `mise run check:fast` | Fast `atla` package check for the inner development loop |
| `mise run lint` | Formatting check plus the CI Clippy policy |
| `mise run deps:check` | Check direct Cargo dependencies with `cargo-machete` |
| `mise run size:bloat` | Report release binary size by crate with `cargo-bloat` |
| `mise run test`, `mise run test:cli`, `mise run test:core`, or `mise run test:e2e` | Workspace or focused test suites |
| `mise run test:nextest` | Parallel workspace unit and integration tests with detailed failure reporting |
| `mise run sccache:stats` | Show cache statistics after opting in with `RUSTC_WRAPPER=sccache CARGO_INCREMENTAL=0` |
| `mise run contract:check` | CLI surface, docs, schemas, and operation-catalog contracts |
| `mise run contract:update` | Regenerate `docs/cli-surface.txt` after a CLI change |
| `mise run tooling:test` | Python maintenance-tool tests |
| `mise run python:lint` / `mise run python:format` | Ruff lint and formatting checks for Python maintenance scripts |
| `mise run python:complexity` | Report-only Python complexity candidates |
| `mise run skill:version` | Exact CLI/skill/Cargo/docs release-version lockstep |
| `mise run security:secrets` | Scan committed and uncommitted source files for secrets |
| `mise run workflow:check` | Validate workflow syntax, immutable Action pins, and release gates |
| `mise run workflow:security` | Audit GitHub Actions workflows with zizmor |
| `mise run deny`, `mise run audit`, or `mise run security` | Dependency policy, RustSec, and combined security checks |
| `mise run coverage` | LCOV generation and the current coverage floor |
| `mise run check:pr` | Sequential local PR gate: secret scan, lint, tests, tooling, MSRV, deny, audit, and workflow security |

After reviewing the tracked `mise.toml`, run `mise trust && mise install` once to provision the
project toolchain. The config pins Node.js for partial-spec filters, Python and Ruff for maintenance
tools, actionlint and zizmor for workflow checks, gitleaks for secret scanning, Cargo dependency/size
analysis, and the Cargo security, coverage, cache, and test tools to versions matching CI where
applicable. `.gitleaks.toml` carries the repository-specific false-positive policy. `deny.toml` rejects unknown registries, Git dependencies,
wildcard dependency versions, unknown licenses, advisories, and yanked crates. Duplicate transitive
versions remain warnings so upgrades can remove them incrementally; do not suppress one without a
documented reason. CI also publishes LCOV and fails below the 53% line-coverage ratchet; raise the
floor only after deterministic tests land.

Generated API code is built into `OUT_DIR`; do not commit it. The generated API manifests
explicitly ignore their generated runtime dependencies for `cargo-machete`, because the references
are emitted outside the source tree. For ordinary CLI/core iteration, `scripts/check-fast.sh`
reuses an opt-in Cargo target cache across worktrees; full PR validation still uses the workspace
commands above. Use `mise run size:bloat` periodically or before a release to inspect the `dist`
profile; it is a diagnostic report, not yet a fixed size budget.

Workflow syntax, immutable Action pins, and workflow security are also checked locally with
`mise run workflow:check` and `mise run workflow:security`; CI runs the same checks in a dedicated
job. Cargo validation uses `--locked` so local and CI dependency resolution cannot drift.

`clippy::too_many_lines` is intentionally not part of the blocking Clippy policy. Its default
threshold flags existing orchestration, formatting, test, and generated functions; without a
baseline or new-code-only gate, enabling it with `-D warnings` would turn a size heuristic into
workspace-wide refactoring debt. Use it as a report/review signal and prefer responsibility and
cognitive-complexity findings when deciding whether a function should be split.

## Code architecture

- `operation/registry.rs` declares typed `OperationId` values, safety metadata, and saved-plan
  route/query contracts together. Runtime command matching may reference those IDs but must not
  introduce duplicate operation strings or plan allowlists.
- One `Invocation` owns immutable global arguments and an invocation-local `OutputSession`.
  Do not add process-global output, receipt, byte-budget, or plan-building state.
- Product dispatchers stay small. Put substantial issue, sprint, page, or blog behavior in the
  action-specific module below its dispatcher rather than adding another large match arm.
- Integration tests are grouped by contract responsibility under `tests/e2e/`; keep the root test
  file as a module index and put shared process/mock-server helpers in `support.rs`.
- `atla-core` keeps generated clients behind `GeneratedTransport` and exposes hand-written domain
  models to the CLI. Generated wire types must not leak into CLI command code.

## Pull requests

Keep changes focused, explain user-visible behavior, and add tests for success and failure paths.
Fill every applicable section of `.github/pull_request_template.md`; explain why any contract or
security checklist item is not applicable. Before opening a PR:

1. run the gitleaks scan, Python Ruff lint/format checks, fmt, Clippy, `cargo machete`, workspace
   tests, RustSec audit, `cargo deny check`, workflow syntax, and workflow-security checks;
2. run `cargo +1.91 check --workspace --all-targets --locked` for changes affecting dependencies/language features;
3. update `CHANGELOG.md` under Unreleased;
4. update every affected document and the agent skill;
5. avoid committing credentials, tenant data, generated SBOMs, or build artifacts.

## CLI surface changes

Any command/flag change under `crates/atla-cli/src/cli/` must follow this order:

1. implement handler and central operation metadata;
2. regenerate `docs/cli-surface.txt`:

   ```bash
   UPDATE_CLI_SURFACE=1 cargo test --locked -p atla cli_surface
   ```

3. update `docs/agent-reference.md`, topic docs, `skills/atla-cli/SKILL.md`, and references;
4. run `cargo test --locked -p atla doc_examples_parse`.

Runnable examples use concrete values. Angle-bracket placeholders belong only in syntax summaries.

## Compatibility and safety

Do not break JSON v1 fields, pagination-token query binding, exit-code meanings, `--yes`, non-TTY
prompt guards, same-origin credentials, method-aware retry, read-only policy, or dry-run no-network
behavior. New JSON fields must be additive and fixtures/schemas must be updated.

Saved apply plans are not arbitrary HTTP requests. New planned operations require an explicit
operation ID, exact local plan construction, route/method/query allowlisting, policy enforcement,
hash/expiry/input/profile/site checks, ambiguity handling, and E2E coverage.

Never add a retry for a non-idempotent mutation after an uncertain timeout/server response. Return
`ambiguous_mutation` and require remote-state verification. Generated Jira/Confluence clients
must stay inside `GeneratedTransport`; execute builders only through `GeneratedTransport::execute`.
The transport is the type-level boundary that applies shared `Retry-After`, backoff, body-reading,
and ambiguity policy.

## API specifications

Refresh with:

```bash
scripts/update-specs.sh
cargo check --workspace --locked
cargo test --workspace --locked
```

Review `specs/PATCHES.md`, operation pruning, manifest hashes/timestamp, and
generated-model conversion tests. The scheduled workflow opens a PR with a generated
partial-spec summary covering operations plus normalized parameter/request/response/schema contract
facts; it never pushes directly to main.
Generate the same summary locally with
`python3 scripts/spec-diff-summary.py --base HEAD` after a refresh.

## Security reports and releases

Report vulnerabilities privately as described in `SECURITY.md`. Do not open a public issue with a
working exploit or credential material.

`release.yml` is intentionally hardened after cargo-dist generation. Follow `docs/releasing.md` and
do not overwrite pinned actions, verified installers, least-privilege permissions, attestations, or
CycloneDX/checksum steps with raw generated output.
