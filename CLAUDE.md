# atla — agent guide

Rust workspace for `atla`, a unified Jira + Confluence Cloud CLI. Primary consumers are
**coding agents**: machine-readable output, stable JSON schemas, and accurate docs are
product features here, not niceties.

## Workspace map

| Crate | Role |
| --- | --- |
| `crates/atla-cli` (package name **`atla`**) | clap definitions (`cli.rs`), command handlers (`commands/`), output rendering, pagination tokens |
| `crates/atla-core` | domain clients + hand-written models (`jira/`, `confluence/`), auth/profiles, markdown⇄ADF (`markdown.rs`) |
| `crates/atla-jira-api`, `atla-confluence-api`, `atla-confluence-v1-api` | progenitor codegen from `specs/*.json` at build time (`build.rs`); generated code is NOT committed |

## Build / test

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets \
  --exclude atla-jira-api --exclude atla-confluence-api --exclude atla-confluence-v1-api -- -D warnings
```

CI (`.github/workflows/ci.yml`) enforces exactly these three. The CLI package is `atla`,
not `atla-cli`: `cargo test -p atla`.

## Changing the CLI surface (checklist)

Any change to commands/flags in `crates/atla-cli/src/cli.rs` MUST be propagated, in this
order. `crates/atla-cli/src/doc_check.rs` enforces steps 2–3 in `cargo test`:

1. Implement the change (`cli.rs` + `commands/`).
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

## Agent-facing contracts (do not break)

- JSON list output shape: `{"<items>": [...], "pagination": {"isLast", "nextPageToken", "nextCommand"}}`.
- `--page-token` is opaque and query-bound; `--all` is mutually exclusive with
  `--limit`/`--page-token`.
- Destructive commands refuse to run without `--yes` (no interactive fallback) — never
  soften this to a prompt.
- Prompts only appear when stdin+stdout are TTYs and `--no-input` is absent; agents must
  never be able to hang on a prompt.
- `confluence page view`/`blog view` return metadata only unless `--format` is given;
  markdown input requires explicit `--representation markdown`.
- Exit-code taxonomy (`crates/atla-cli/src/error.rs`): 2 usage, 3 auth, 4 not-found,
  5 retryable, 1 other; `-o json` emits `{"error": {...}}` on stderr. Documented in
  `docs/agent-reference.md` §3 — keep all three in sync.

## Known debt (verified 2026-07, keep in mind when touching these areas)

- Generated-client errors flow through `ProgenitorResultExt::or_api_error()`
  (`atla-core/src/generated_api.rs`) which reads the response body. Never map errors
  with a sync helper that drops the body.
- Retry on 429/502/503/504 (`send_with_retry`, `client.rs`) covers only the raw-reqwest
  paths (`read_json`/`read_empty`: Agile API, attachments, user search). The progenitor
  clients accept a plain `reqwest::Client`, so their calls are NOT retried — callers
  branch on exit code 5 / `ApiError::retryable()`.
- Shared Basic-auth client construction lives in `AtlassianClient::authed_http_client()`;
  don't hand-roll header setup in Jira/Confluence client constructors.
