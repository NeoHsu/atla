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
- After any spec refresh, re-apply every patch in `specs/PATCHES.md` by hand — upstream
  enums drift from real API responses (two past deserialization breakages).
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

## Known debt (verified 2026-07, keep in mind when touching these areas)

- `generated_error` (33 call sites, `atla-core/src/{jira,confluence}/util.rs`) drops API
  error bodies — 400s surface as empty messages. Prefer `generated_error_with_body`; a
  proper fix should route all errors through `extract_api_error_body` (`client.rs`).
- All runtime failures exit 1; no exit-code taxonomy, no JSON error output, no 429 retry.
- Most clap args lack doc comments → empty `--help`. When touching a command, add them.
- `atla-confluence-api` compiles a 103k-line client from the full v2 spec; a partial-spec
  filter script (like the jira one) would cut clean builds drastically.
