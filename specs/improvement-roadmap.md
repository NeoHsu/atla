# Improvement roadmap

Updated 2026-07-21 for the 0.6 release line. This file tracks only
current work; completed review items belong in `CHANGELOG.md` and Git history.

## Release gates for 0.6.0

- Review and merge PR #4 through CI; do not push the feature branch directly
  to `main`.
- Cut the combined `[Unreleased]` changelog into a dated `[0.6.0]` section
  only in the reviewed release commit.
- Require `python3 scripts/check-skill-version.py --tag v0.6.0` to confirm exact
  CLI/skill/lockfile/docs version lockstep before tagging.
- Run the local/global cargo-dist artifact smoke tests and
  `python3 scripts/verify-release-artifacts.py` before tagging.
- Complete the bounded Jira + Confluence sandbox ledger from
  `docs/live-smoke.md`; classify every selected operation and clean or report
  every temporary resource.
- Create and push a signed `v0.6.0` tag only from the clean reviewed release
  commit. The existing `v0.9.0` prerelease is unrelated and must not be moved.

## Remaining engineering backlog

### Raise deterministic network-path coverage

CI now fails below 53% line coverage. Ratchet that floor upward only after
adding wiremock tests for thin core modules, especially Confluence
comments/labels/search/spaces and Jira comments/projects/sprints. Prefer
behavior assertions for retry, error-body propagation, pagination, and
mutation ambiguity over line-only tests.

### Deeper output snapshots

The E2E suite locks representative JSON/table/CSV/keys output and the
CLI/schema surfaces. Extend snapshot coverage to sprint, page, blog, and space
printers when those contracts change. Avoid unstable timestamps or request IDs.

### Behavioral skill evaluation

The bundled skill has deterministic syntax and form checks plus adversarial
eval definitions. Run mutation-bearing evals only against an explicitly
approved sandbox, or add a fake `atla` harness so fail-closed target selection
and cleanup can be compared without tenant writes.

## Product design deferred to a breaking release

`confluence page view` and `blog view` remain metadata-only by default in 0.6
to preserve bounded agent output and existing scripts. Metadata JSON is
self-describing (`bodyIncluded` and `bodyCommand`), `--metadata-only` is
explicit, and callers can use `--format`, `--fields`, and `--max-chars`.
Reconsider defaulting to Markdown only for a major compatibility boundary and
only with a conservative default body limit.

## Completed in the 0.6 hardening work

- Partial OpenAPI refreshes are reproducible and now report parameter,
  request/response, required, enum, and nested schema contract facts rather
  than only operation/schema counts.
- Generated Jira/Confluence requests share bounded method-aware retry,
  exponential backoff, and `Retry-After` handling with raw reqwest paths;
  uncertain mutations remain non-retryable.
- Confluence page/blog dry-runs produce exact local converted payloads without
  network access.
- Confluence body views support explicit metadata mode, top-level JSON field
  projection, and Unicode-safe rendered-body limits. Likely Markdown sent as
  storage produces a warning instead of being silently misinterpreted.
- Stable Confluence space JSON exposes the optional `spaceOwnerId` returned by
  v2.
- CLI and bundled skill releases use exact SemVer lockstep, tag-pinned skill
  installation, and a fail-closed local version gate with actionable remediation.
- The obsolete unbudgeted `scripts/adf_spec_validate.py` live mutation helper
  was removed; its ADF cases are covered by local Markdown/ADF tests and the
  bounded live-smoke workflow.
- CI uses the SHA-pinned Node 24 checkout action and enforces a coverage floor.
