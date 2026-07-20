# Contributing to atla

Thank you for improving atla. The CLI is consumed by automation and coding agents, so deterministic
behavior and machine-readable contracts are part of the public API.

## Development setup

Install Rust 1.91 or newer, then run:

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets \
  --exclude atla-jira-api --exclude atla-confluence-api --exclude atla-confluence-v1-api -- -D warnings
cargo audit
cargo deny check
```

`deny.toml` rejects unknown registries, Git dependencies, wildcard dependency versions, unknown
licenses, advisories, and yanked crates. Duplicate transitive versions remain warnings so upgrades
can remove them incrementally; do not suppress one without a documented reason.

Generated API code is built into `OUT_DIR`; do not commit it.

## Pull requests

Keep changes focused, explain user-visible behavior, and add tests for success and failure paths.
Before opening a PR:

1. run fmt, Clippy, workspace tests, RustSec audit, and `cargo deny check`;
2. run `cargo +1.91 check --workspace` for changes affecting dependencies/language features;
3. update `CHANGELOG.md` under Unreleased;
4. update every affected document and the agent skill;
5. avoid committing credentials, tenant data, generated SBOMs, or build artifacts.

## CLI surface changes

Any command/flag change in `crates/atla-cli/src/cli.rs` must follow this order:

1. implement handler and central operation metadata;
2. regenerate `docs/cli-surface.txt`:

   ```bash
   UPDATE_CLI_SURFACE=1 cargo test -p atla cli_surface
   ```

3. update `docs/agent-reference.md`, topic docs, `skills/atla-cli/SKILL.md`, and references;
4. run `cargo test -p atla doc_examples_parse`.

Runnable examples use concrete values. Angle-bracket placeholders belong only in syntax summaries.

## Compatibility and safety

Do not break JSON v1 fields, pagination-token query binding, exit-code meanings, `--yes`, non-TTY
prompt guards, same-origin credentials, method-aware retry, read-only policy, or dry-run no-network
behavior. New JSON fields must be additive and fixtures/schemas must be updated.

Saved apply plans are not arbitrary HTTP requests. New planned operations require an explicit
operation ID, exact local plan construction, route/method/query allowlisting, policy enforcement,
hash/expiry/input/profile/site checks, ambiguity handling, and E2E coverage.

Never add a retry for a non-idempotent mutation after an uncertain timeout/server response. Return
`ambiguous_mutation` and require remote-state verification.

## API specifications

Refresh with:

```bash
scripts/update-specs.sh
cargo check --workspace
cargo test --workspace
```

Review `specs/PATCHES.md`, operation pruning, manifest hashes/timestamp, and generated-model
conversion tests. The scheduled workflow opens a PR; it never pushes directly to main.

## Security reports and releases

Report vulnerabilities privately as described in `SECURITY.md`. Do not open a public issue with a
working exploit or credential material.

`release.yml` is intentionally hardened after cargo-dist generation. Follow `docs/releasing.md` and
do not overwrite pinned actions, verified installers, least-privilege permissions, attestations, or
CycloneDX/checksum steps with raw generated output.
