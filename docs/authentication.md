---
title: Authentication
description: Manage profiles, tokens, and credential storage for atla.
---

# Authentication

atla uses API tokens to authenticate with Atlassian Cloud instances. Credentials are stored per-profile and can be managed via the OS keyring or a local file.

Atlassian API tokens expire after a configurable 1–365 days. Record the selected expiry and
rotate the stored token before that date. Unscoped tokens use the
`https://SITE.atlassian.net` URL. For scoped tokens, store the tenant cloud ID in the profile;
atla routes Jira and Confluence through their product-specific
`api.atlassian.com/ex/{product}/{cloudId}` gateways.

---

## Command reference

### `atla auth login`

Authenticate with an Atlassian instance and store credentials.

```
atla auth login [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--instance <URL>` | Atlassian instance URL (e.g. `https://example.atlassian.net`) |
| `--cloud-id <ID>` | Tenant cloud ID; enables product-specific scoped-token gateway URLs |
| `--email <EMAIL>` | Account email address |
| `--token <TOKEN>` | API token (visible in process arguments; prefer prompt/env/stdin) |
| `--token-stdin` | Read one token line from stdin without exposing it in process arguments |
| `--storage <METHOD>` | Credential storage: `keyring` (default) or `file` |

**Interactive mode** (default when stdin is a terminal):

```bash
atla auth login
# Prompts for instance, email, and token interactively
```

**Non-interactive mode** (stdin avoids shell history and process arguments):

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login \
  --no-input \
  --instance https://example.atlassian.net \
  --email you@example.com \
  --token-stdin \
  --storage keyring
```

**Scoped token** (the site URL remains the browser/web-link origin):

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login \
  --instance https://example.atlassian.net \
  --cloud-id 11111111-2222-3333-4444-555555555555 \
  --email you@example.com \
  --token-stdin
```

Discover the cloud ID and derived endpoints without credentials:

```bash
atla --output json auth discover --site https://example.atlassian.net
```

Omit `--cloud-id` for an unscoped token. To switch an existing profile back to site routing:

```bash
atla config set cloud-id ""
```

**Dry run** (preview without saving):

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --instance https://example.atlassian.net \
  --email you@example.com --token-stdin --dry-run
```

Instance URL normalization:

- `example.atlassian.net` → `https://example.atlassian.net`
- `https://example.atlassian.net/` → `https://example.atlassian.net`

---

### `atla auth status`

Display the current authentication state.

```bash
atla auth status
```

Shows:

- Active profile name
- Instance URL
- Email
- Credential storage method
- API target (`site` or `scoped-token-gateway`) and cloud ID when configured
- Profile operation-policy mode
- Whether a valid token is available

---

### `atla auth logout`

Remove stored credentials for the current profile. This is destructive and requires `--yes`.

```bash
atla auth logout --yes
```

---

### `atla auth switch`

Switch the active profile.

```bash
atla auth switch <profile>
```

Example:

```bash
atla auth switch personal
```

This updates `default.profile` in the config file.

---

## Multiple profiles

You can maintain separate profiles for different Atlassian instances (e.g. work and personal, or production and staging).

### Creating a named profile

Log in with a different instance to create a new profile:

```bash
printf '%s\n' "$WORK_TOKEN" | atla auth login --profile work \
  --instance https://work.atlassian.net --email work@company.com --token-stdin
```

```bash
printf '%s\n' "$PERSONAL_TOKEN" | atla auth login --profile personal \
  --instance https://personal.atlassian.net --email me@personal.com --token-stdin
```

Each unique instance/email combination creates a separate profile in your config.

### Switching profiles

Set the default profile:

```bash
atla auth switch work
```

### Per-command profile override

Use `--profile` to run a single command against a different profile without switching:

```bash
atla jira search "project = SIDE" --profile personal
```

---

## Token storage strategies

### Keyring (default)

```bash
atla auth login --storage keyring
```

Tokens are stored in the OS credential manager:

- **macOS**: Keychain
- **Linux**: Secret Service (GNOME Keyring / KWallet)
- **Windows**: Credential Manager

Best for: developer workstations with a desktop environment.

### File storage

```bash
atla auth login --storage file
```

Tokens are stored in `~/.config/atla/credentials.toml` (plain text, file-permission protected).

Best for:

- Headless servers and CI/CD
- Containers without a keyring daemon
- Environments where D-Bus / Secret Service is unavailable

The credentials file location can be overridden:

```bash
export ATLA_CREDENTIALS=/path/to/credentials.toml
```

---

## Environment variable overrides

Environment variables take precedence over stored credentials, regardless of storage method.

| Variable | Description |
|----------|-------------|
| `ATLA_TOKEN` | API token (highest priority) |
| `ATLA_API_TOKEN` | API token (alias, same behavior) |

Example for CI pipelines:

```bash
export ATLA_TOKEN="$ATLASSIAN_API_TOKEN"
atla jira search "project = PROJ"
```

When an environment variable is set, atla uses it directly and does not read from keyring or file storage.

---

## Config schema and safe writes

Current config files use `schema_version = 2`. When atla first opens a legacy unversioned
config, it creates `config.toml.v1.bak` before atomically migrating the original. Config and
file-backed credential updates are written through a same-directory temporary file, synced,
and atomically replaced. On Unix, files are mode `0600`.

---

## The `--dry-run` flag

Use `--dry-run` with auth commands to preview what would happen without making changes:

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --instance https://example.atlassian.net \
  --email you@example.com --token-stdin --dry-run
```

Output shows the profile that would be created/updated, the storage method, and the normalized instance URL — but nothing is persisted.

---

## Troubleshooting

Start with a local, redacted diagnostic report:

```bash
atla doctor --output json
```

It checks the config path/schema, selected profile, credential source, site URL, API target, and
policy without printing the token. If local checks are sound, explicitly add `--network` to verify
the unauthenticated tenant-info endpoint and discovered cloud ID:

```bash
atla doctor --network --timeout 10 --output json
```

### Token not found

```
Error: No token available for profile "work"
```

**Causes:**

- The profile was created but `login` was not completed
- Keyring entry was deleted externally
- Environment variable is not set

**Fix:**

```bash
printf '%s\n' "$NEW_TOKEN" | atla auth login --instance https://example.atlassian.net \
  --email you@example.com --token-stdin
```

### Keyring unavailable

```
Error: Failed to access system keyring
```

**Causes:**

- Running in a headless environment (SSH, container, CI)
- D-Bus or Secret Service daemon is not running (Linux)

**Fix:** Switch to file-based storage:

```bash
atla auth login --storage file
```

Or use an environment variable:

```bash
export ATLA_TOKEN="your-api-token"
```

### Wrong profile active

If commands hit the wrong instance:

```bash
# Check which profile is active
atla auth status

# Switch to the correct one
atla auth switch <profile>

# Or use --profile for a one-off command
atla jira issue view PROJ-1 --profile work
```

### Token expired or revoked

Atlassian API tokens expire on the date selected at creation (1–365 days) and can also be
revoked earlier. Tokens created before the expiration policy was introduced were assigned an
expiry between March 14 and May 12, 2026. If requests return 401:

1. Check whether the token expired or was revoked.
2. Generate a replacement at <https://id.atlassian.com/manage-profile/security/api-tokens>.
3. Re-run `atla auth login` with the replacement token.

---

## See also

- [Getting Started](./getting-started.md) — installation and first-time setup
- [Configuration](./configuration.md) — config keys, aliases, and environment variables
