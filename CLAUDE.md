# atla — agent guide

Rust workspace for `atla`, a unified Jira + Confluence Cloud CLI. Primary consumers are
**coding agents**: machine-readable output, stable JSON schemas, and accurate docs are
product features here, not niceties.

## Workspace map

| Crate | Role |
| --- | --- |
| `crates/atla-cli` (package name **`atla`**) | clap definitions (`cli/`), command handlers (`commands/`), output rendering, pagination tokens |
| `crates/atla-core` | domain clients + hand-written models (`jira/`, `confluence/`), auth/profiles, markdown⇄ADF (`markdown/`) |
| `crates/atla-jira-api`, `atla-confluence-api`, `atla-confluence-v1-api` | progenitor codegen from `specs/*.json` at build time (`build.rs`); generated code is NOT committed |

## Build / test

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets \
  --exclude atla-jira-api --exclude atla-confluence-api --exclude atla-confluence-v1-api -- -D warnings
```

CI (`.github/workflows/ci.yml`) enforces these three plus a RustSec dependency audit. The CLI
package is `atla`, not `atla-cli`: `cargo test -p atla`.

## Changing the CLI surface (checklist)

Any change to commands/flags under `crates/atla-cli/src/cli/` MUST be propagated, in this
order. `crates/atla-cli/src/doc_check.rs` enforces steps 2–3 in `cargo test`:

1. Implement the change (`cli/` + `commands/`).
2. Regenerate the surface snapshot: `UPDATE_CLI_SURFACE=1 cargo test -p atla cli_surface`
   (updates `docs/cli-surface.txt`; the test fails until you do this).
3. Update every doc that mentions the command — all `atla` examples in these files are
   parse-checked against the real clap definition by the `doc_examples_parse` test:
   - `docs/agent-reference.md` (§2 command tree + §4/§5 flag tables)
   - `docs/jira.md` / `docs/confluence.md` / other `docs/*.md`
   - `skills/atla-cli/SKILL.md` + `skills/atla-cli/references/*.md`
4. In docs, write runnable examples with concrete values (`--version 4`, not
   `--version N`); use `<ANGLE>` placeholders only in usage-summary lines.

## Skill sync rules

- `skills/atla-cli/` in this repo is the **single source of truth** for the agent skill.
- The installed skill (`~/.agents/skills/atla-cli`) must be a **symlink** to this repo's
  `skills/atla-cli`. Never edit the installed location directly; never install with
  `npx skills add --copy` (a copy caused a 3-week doc/CLI drift in 2026-05/06).
- Skill content policy: SKILL.md holds the command tree, common traps, and quick
  patterns; exact flag syntax lives in `references/*.md`.
- The distributable skill must remain self-contained around the installed `atla` CLI. Do not
  reference or bundle repository-only maintainer, CI, release, or live-smoke tooling there;
  document those workflows under `docs/` instead.

## API codegen / specs

- Refresh flow: `scripts/update-specs.sh` then `cargo build` (each API crate's `build.rs`
  regenerates its client). There is no `generate.sh`.
- All three clients build from **partial specs** (`specs/*-partial.json`). The jira and
  confluence-v1 scripts hand-build minimal specs; `confluence-v2-partial-spec.js` prunes
  the upstream spec to the used operations via $ref closure — when core starts calling a
  new v2 operation, add its snake_case name to `usedOperations` in that script and rerun.
- Enum patches (`specs/PATCHES.md`) for confluence-v2 are applied automatically by the
  filter script (`stripEnumSchemas`); the jira patch is still manual — check PATCHES.md
  on every refresh.
- `specs/manifest.json` tracks spec sources + SHA256; keep it updated via the script.

## Release workflow hardening

- `.github/workflows/release.yml` starts from cargo-dist output but is intentionally post-generated:
  action refs and installer bytes are SHA-pinned, permissions are job-scoped, shell expressions are
  injection-safe, and cargo-cyclonedx 0.5.9 emits a binary-only CycloneDX 1.5 SBOM with hashes.
- `allow-dirty = ["ci"]` in `dist-workspace.toml` is intentional. Do not replace release.yml with
  raw `dist generate` output. If regenerating, reapply and verify the hardening with pi-lens before
  committing, then run `dist plan` and artifact smoke tests.

## Agent-facing contracts (do not break)

- JSON object output includes additive `"schemaVersion": 1`; list shape remains
  `{"schemaVersion": 1, "<items>": [...], "pagination": {"isLast", "nextPageToken", "nextCommand"}}`.
- `--page-token` is opaque and query-bound; `--all` is mutually exclusive with
  `--limit`/`--page-token`. A global `--max-pages`/`--max-items` budget makes `--all`
  resumable and must preserve next-page metadata.
- Destructive commands refuse to run without `--yes` (no interactive fallback) — never
  soften this to a prompt. `--read-only` classification comes from `operation.rs` and must
  reject real local or remote mutations before credentials/network access; mutation dry-run
  previews remain allowed. Profile policy uses deny → allow → mode precedence.
- Prompts only appear when stdin+stdout are TTYs and `--no-input` is absent; agents must
  never be able to hang on a prompt.
- Saved plans are allowlisted semantic operations, not arbitrary HTTP replay. Apply must validate
  hash, expiry, inputs, profile/site, policy, same-origin route/method/query, and `--yes` before
  token/network access.
- `confluence page view`/`blog view` return self-describing metadata only unless `--format` is
  given; `--metadata-only` makes that choice explicit, `--fields` projects top-level JSON fields,
  and `--max-chars` bounds rendered bodies. Markdown input requires explicit
  `--representation markdown`; likely Markdown sent as storage emits a warning.
- Exit-code taxonomy (`crates/atla-cli/src/error.rs`): 2 usage, 3 auth, 4 not-found,
  5 safe-to-retry, 1 other/ambiguous mutation; `-o json` emits `{"error": {...}}` on
  stderr. Never mark an uncertain mutation retryable. Documented in
  `docs/agent-reference.md` §3 — keep all three in sync.
- Config schema version 2 auto-migrates legacy files only after creating a `.v1.bak`; config
  and file credentials use atomic mode-0600 replacement. Scoped profiles retain the site URL
  but route each product through `api.atlassian.com/ex/{product}/{cloudId}`.

## Known debt (verified 2026-07, keep in mind when touching these areas)

- Every generated-client call must flow through `generated_api::generated_request(method, ...)`.
  It reads final error bodies, honors `Retry-After`/backoff, retries only safe methods except an
  explicit 429 rejection, and leaves uncertain mutations non-retryable. Never call a generated
  builder's `.send()` directly or map errors with the sync fallback.
- Shared Basic-auth client construction lives in `AtlassianClient::authed_http_client()`;
  don't hand-roll header setup in Jira/Confluence client constructors.
