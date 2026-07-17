---
title: Config Migration to Schema v2
description: Backward-compatible migration, backups, and scoped-token fields.
---

# Config migration to schema v2

Schema v2 adds an explicit version, optional cloud ID, and per-profile operation policy while
retaining all v1 fields.

```toml
schema_version = 2

[profiles.work]
instance = "https://example.atlassian.net"
email = "you@example.com"
credential_store = "keyring"
cloud_id = "11111111-2222-3333-4444-555555555555"

[profiles.work.policy]
mode = "read-only"
deny = ["*.delete"]
```

## Automatic migration

An unversioned config is interpreted as schema v1. On a normal load, atla:

1. parses and validates the complete old file;
2. writes the untouched source to `config.toml.v1.bak` if that backup does not exist;
3. serializes schema v2 to a mode-`0600` same-directory temporary file;
4. flushes and syncs it;
5. atomically replaces `config.toml`.

With global `--read-only`, migration happens only in memory and neither the config nor a backup is
written.

Tokens are not moved: credential identity remains profile name + email + site URL, and token
material stays in the configured keyring/file store.

## Compatibility and rollback

Profiles without `cloud_id` continue using the site URL exactly as before. Existing defaults and
aliases are preserved. To roll back, stop atla, inspect the backup, and replace `config.toml` with
`config.toml.v1.bak`. Older atla versions do not reliably ignore the schema field or new nested policy fields, so do
not share one writable config between old and new binaries.

To disable scoped routing without changing credentials:

```bash
atla config set cloud-id ""
```
