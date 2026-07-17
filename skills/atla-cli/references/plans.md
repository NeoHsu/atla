# Saved Plan Reference

Use `atla plan --help` and `atla apply --help` as the runtime syntax authority. Saved plans are
available only for the operation IDs listed below and always use JSON output.

## Supported operations

- `jira.issue.create`
- `confluence.page.create`
- `confluence.page.update`
- `confluence.blog.create`
- `confluence.blog.update`

## Generate

```bash
atla plan jira issue create --project PROJ --type Task --summary 'Prepare launch' \
  --out create-plan.json

atla plan confluence page create --space-id 123 --title 'Runbook' \
  --body-file runbook.md --representation markdown --out page-plan.json
```

Plan generation is local and network-free, so values that normally require a lookup must be
explicit:

- Confluence create plans require `--space-id`; `--space` cannot be resolved offline.
- Page/blog update plans require `--title`, `--body` or `--body-file`, and the explicit next
  `--version`.
- `--resolve-mentions` is unavailable offline; use deterministic `--mention NAME=ACCOUNT_ID` on
  page operations.

Optional `--expires-in 1800` accepts 1–86,400 seconds and defaults to 3,600. A plan file is at most
1 MiB and is written atomically with user-only permissions on Unix. It contains the exact URL/body,
profile/site, preconditions, input-file hashes, expiration, and `planHash`; it excludes credentials.

`--read-only` allows a stdout JSON dry-run preview but blocks writing a saved plan file. Operations
outside the supported list reject `--dry-run --output json`; use a non-JSON dry-run for those.

## Review

Before apply, verify:

1. `operation`, `profile`, and `site` match the approved target.
2. Every request uses the expected method, same-origin URL, path, and query.
3. `unresolved` is empty and input-file hashes still match.
4. The body and expected effect match the user's request.

A SHA-256 plan digest is tamper-evident, not a signature. Never apply a plan from an untrusted
source.

## Apply

```bash
atla apply create-plan.json --yes --output json
```

Apply requires `--yes`; there is no prompt. It validates schema, digest, expiry, input hashes,
active profile/site, operation policy, same-origin URL, and a built-in method/path/query allowlist.
It is not arbitrary HTTP replay, and `--read-only` blocks it.

Successful output includes mutation-receipt metadata. On `ambiguous_mutation`, query the target to
determine whether it committed before considering another apply.
