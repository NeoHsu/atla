# Improvement roadmap

Consolidated from a deep architecture / agent-UX / doc-drift review on 2026-07-07
(atla 0.3.0, commit da5e667). Ordered by return on investment. Each item is scoped so a
single focused session can ship it. Check items off (or delete them) as they land.

## P0 — error contract for machine users

The single biggest gap for the primary (agent) audience.

1. **Stop dropping API error bodies.** `generated_error` maps
   `Error::UnexpectedResponse` to `body: String::new()`
   (`crates/atla-core/src/jira/util.rs` ~L47, `confluence/util.rs` ~L78), so agents see
   `Atlassian API returned 400 Bad Request: ` with nothing after the colon — including
   for JQL syntax errors that Jira explains precisely. 33 call sites use it vs 15 for
   `generated_error_with_body`. Fix: make the body-reading path the only path and feed it
   through `extract_api_error_body` (`atla-core/src/client.rs:155-190`, already good).
2. **Fix the network-error misclassification.** `CommunicationError` currently becomes
   `ApiError::Decode` ("failed to decode response") in all three `generated_error*`
   copies — misleading and kills retry logic. Add an `ApiError::Network` variant.
3. **Exit-code taxonomy + JSON errors.** Everything exits 1 today
   (`crates/atla-cli/src/error.rs` is a 5-line stub). Define
   `CliError { kind, status, retryable, message, hint }`; exit 2=usage, 3=auth,
   4=not-found, 5=rate-limited/retryable, 1=other; with `-o json`, emit
   `{"error": {...}}` on stderr. Document the taxonomy in `docs/agent-reference.md` and
   the skill.

## P1 — test net for the agent contract

4. **Snapshot-test output rendering** (`insta`): table/json/csv/keys for the main
   printers in `commands/jira/format.rs` (1493 lines, 3 tests) and
   `commands/confluence/format.rs` (1109 lines, 8 tests). The JSON `pagination` object is
   an API contract with zero lock-in today.
5. **HTTP-level tests** (`wiremock`) for `JiraClient`/`ConfluenceClient`: error mapping
   (the P0 fixes need regression cover) and the pagination accumulation loop
   (`jira/util.rs:17-33`).
6. **`trycmd` end-to-end runs of `--dry-run`** — dry-run needs no network and exercises
   arg parsing, profile resolution, and request construction.

## P2 — help text & discoverability

7. Almost every subcommand/arg in `cli.rs` lacks a doc comment → empty `--help`; agents
   cannot discover `--field NAME=VALUE` format or that `--representation markdown`
   exists. Add doc comments everywhere plus `after_help` examples on the big four
   (`issue create`, `search`, `page create`, `comment add`). Zero risk, high payoff.
8. Print the request **payload** on `--dry-run`, not just method+URL
   (`commands/jira/issue.rs:360-368`) — agents use dry-run to verify `--field` assembly.

## P3 — build & duplication debt

9. **Trim the confluence v2 spec.** The generated client is 103k lines (jira: 5.7k)
   because no partial-spec filter exists for it. Clone
   `scripts/jira-v3-partial-spec.js` → `confluence-v2-partial-spec.js`; while there,
   encode the `specs/PATCHES.md` enum-stripping rules into the filter so
   `update-specs.sh` becomes one-shot idempotent.
10. **Deduplicate** the three `generated_error*` copies (macro over the three progenitor
    error types) and the two hand-rolled authed `reqwest::Client` builders
    (`jira/mod.rs:44-62`, `confluence/mod.rs:26-50` — includes runtime `expect()`s);
    hang a shared builder off `AtlassianClient`.
11. **Retry on 429/5xx** with `Retry-After` support at the shared client layer (natural
    follow-up to #10).

## P4 — agent ergonomics (design decisions, discuss before doing)

12. `confluence page view --format` default: consider defaulting to `markdown` (or add
    `--metadata-only`) — today's metadata-only default surprises agents. Breaking-ish.
13. `--representation` default is `storage`; feeding markdown without the flag silently
    produces broken content. Consider sniffing + warning.
14. Confluence-side `--fields` filtering and a `--max-chars` guard on large page bodies
    (protects agent context windows).
15. `confluence page comment add` lacks `--body` while `jira issue comment add` has it —
    align (agents guess by analogy). If added, also update the "Common Traps" section in
    `skills/atla-cli/SKILL.md`.

## Done (this review)

- Doc-drift protection: `crates/atla-cli/src/doc_check.rs` — every `atla` example in
  docs/skills parse-checked; CLI surface snapshotted to `docs/cli-surface.txt`.
- Fixed wrong examples: `issue get`→`view` (docs/authentication.md,
  docs/configuration.md), `-o`→`--save-to` (agent-reference §5), confluence comment
  `--body`→positional (SKILL.md).
- agent-reference: added `issue fields`, `github-links`/`github-commits`,
  `blog comment delete`, `--with-github`, `--with-attachments`, `blog view --format`.
- SKILL.md: "Common Traps" section; completion shell list corrected.
- Installed skill re-linked as symlink to repo (was a stale copy).
- `CLAUDE.md` with the CLI-surface change checklist.
