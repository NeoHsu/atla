## Summary

<!-- What user or maintainer problem does this solve? -->

## Changes

-

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --exclude atla-jira-api --exclude atla-confluence-api --exclude atla-confluence-v1-api -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo +1.91.0 check --workspace` when dependencies or language features change
- [ ] `cargo audit`
- [ ] `cargo deny check`

## Public contract review

- [ ] CLI commands and flags are unchanged, or `docs/cli-surface.txt`, topic docs, and `skills/atla-cli/` were updated.
- [ ] Stdout JSON is unchanged, or its schema and fixture were updated additively.
- [ ] Stderr JSON error shape and exit-code classification are unchanged, or the compatibility impact is documented.
- [ ] Pagination token binding and resume behavior are unchanged, or migration notes were added.
- [ ] Mutation receipt, plan, dry-run, `--yes`, and `--read-only` guarantees remain intact.
- [ ] New operations are registered with risk, HTTP method, pagination, dry-run, and policy metadata.

## Security and operations

- [ ] No credentials, tenant data, generated build output, live-smoke ledger, or temporary resource IDs are committed.
- [ ] New network paths preserve same-origin authentication, bounded timeouts, and mutation retry safety.
- [ ] New mutations include mock E2E coverage and a documented verification/cleanup path.

## Documentation and release notes

- [ ] `CHANGELOG.md` was updated under **Unreleased**.
- [ ] User, maintainer, and agent-skill documentation was updated where applicable.
- [ ] Not applicable items above are explained in the summary.
