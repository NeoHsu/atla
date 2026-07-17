# Saved Plan Reference

## Generate

```bash
atla plan jira issue create --project PROJ --type Task --summary 'Prepare launch' \
  --out create-plan.json

atla plan confluence page create --space-id 123 --title 'Runbook' \
  --body-file runbook.md --representation markdown --out page-plan.json
```

Optional: `--expires-in 1800` (range 1–86,400 seconds; default 3,600).

Supported IDs: `jira.issue.create`, `confluence.page.create`, `confluence.page.update`,
`confluence.blog.create`, and `confluence.blog.update`.

Plan generation is local and network-free. The JSON includes exact URL/body, profile/site,
preconditions, input-file hashes, expiration, and `planHash`; it excludes credentials.

## Apply

```bash
atla apply create-plan.json --yes --output json
```

Apply requires `--yes` and validates schema, digest, expiry, input hashes, profile/site, original
operation policy, same-origin URL, and a built-in method/path/query allowlist. It is not arbitrary
HTTP replay. `--read-only` blocks apply.

A digest is not a signature. Review the plan and never apply one from an untrusted source.
Successful JSON includes mutation receipt fields. On `ambiguous_mutation`, verify remote state and
do not blindly apply again.
