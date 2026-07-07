# Improvement roadmap

Consolidated from a deep architecture / agent-UX / doc-drift review on 2026-07-07
(atla 0.3.0, commit da5e667). Ordered by return on investment. Each item is scoped so a
single focused session can ship it. Check items off (or delete them) as they land.

## Remaining backlog

- **Deeper output snapshot coverage**: the e2e suite locks keys/csv/table/json for
  search and boards; extending the same wiremock pattern to sprint/page/space printers
  is mechanical when a regression appears.
- **Retry for generated-client calls**: progenitor clients accept a plain
  `reqwest::Client`, so `send_with_retry` cannot wrap them. Options: progenitor
  `pre_hook`/custom-client support in a future version, or operation-level retry
  wrappers in core. Until then only raw paths (Agile API, attachments) auto-retry.
- **Confluence dry-run payload preview**: body conversion happens after the dry-run
  check and may need the network (space-id resolution, `--resolve-mentions`), so page
  create/update previews print URL only. Printing the local-only conversion result
  would need the markdown pipeline hoisted above the dry-run check.

## P4 — agent ergonomics (design decisions, discuss before doing)

12. `confluence page view --format` default: consider defaulting to `markdown` (or add
    `--metadata-only`) — today's metadata-only default surprises agents. Breaking-ish.
13. `--representation` default is `storage`; feeding markdown without the flag silently
    produces broken content. Consider sniffing + warning.
14. Confluence-side `--fields` filtering and a `--max-chars` guard on large page bodies
    (protects agent context windows).
## Done (this review)

- **P1 test net (2026-07-07):** e2e suite (`crates/atla-cli/tests/e2e.rs`) runs the real
  binary against wiremock — exit codes, error bodies, JSON errors, output formats,
  pagination accumulation, dry-run isolation, and 429 retry.
- **P2 discoverability (2026-07-07):** every subcommand/arg documented in `--help`,
  after_help examples on the big five; `--dry-run` prints the JSON request body for
  Jira issue create/update and comment add (`request_body()` previews live in core).
- **P3 build debt (2026-07-07):** confluence-v2 now builds from a pruned partial spec
  (103k → 34k generated lines; `scripts/confluence-v2-partial-spec.js`, $ref-closure
  filter with automatic enum stripping); Basic-auth client construction unified into
  `AtlassianClient::authed_http_client()`; raw-path requests retry transient failures
  with Retry-After support (`send_with_retry`).
- **P4-15 (2026-07-07):** `confluence page/blog comment add` accept `--body`, matching
  the jira command.

- **P0 error contract (all three items, verified against the live API 2026-07-07):**
  API error bodies now surface (bad JQL returns Jira's exact explanation); all
  generated-client errors flow through `ProgenitorResultExt::or_api_error()` in the new
  `atla-core/src/generated_api.rs` (also unified the three per-crate copies);
  `ApiError::Network` fixes the CommunicationError→Decode misclassification and
  `ApiError::retryable()/status()` expose retry semantics; exit codes are classified
  (2 usage / 3 auth / 4 not-found / 5 retryable / 1 other, `atla-cli/src/error.rs`) and
  `-o json` emits a structured `{"error": {...}}` on stderr.

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
