---
title: Authentication
description: Manage profiles, tokens, and credential storage for atla.
---

# Authentication

atla uses API tokens to authenticate with Atlassian Cloud instances. Credentials are stored per-profile and can be managed via the OS keyring or a local file.

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
| `--email <EMAIL>` | Account email address |
| `--token <TOKEN>` | API token |
| `--storage <METHOD>` | Credential storage: `keyring` (default) or `file` |

**Interactive mode** (default when stdin is a terminal):

```bash
atla auth login
# Prompts for instance, email, and token interactively
```

**Non-interactive mode** (all flags provided, or `--no-input`):

```bash
atla auth login \
  --instance https://example.atlassian.net \
  --email you@example.com \
  --token YOUR_API_TOKEN \
  --storage keyring
```

**Dry run** (preview without saving):

```bash
atla auth login --instance https://example.atlassian.net --email you@example.com --token TOKEN --dry-run
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
- Whether a valid token is available

---

### `atla auth logout`

Remove stored credentials for the current profile.

```bash
atla auth logout
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
atla auth login \
  --instance https://work.atlassian.net \
  --email work@company.com \
  --token WORK_TOKEN
```

```bash
atla auth login \
  --instance https://personal.atlassian.net \
  --email me@personal.com \
  --token PERSONAL_TOKEN
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

## The `--dry-run` flag

Use `--dry-run` with auth commands to preview what would happen without making changes:

```bash
atla auth login \
  --instance https://example.atlassian.net \
  --email you@example.com \
  --token TOKEN \
  --dry-run
```

Output shows the profile that would be created/updated, the storage method, and the normalized instance URL — but nothing is persisted.

---

## Troubleshooting

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
atla auth login --instance https://example.atlassian.net --email you@example.com --token NEW_TOKEN
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

Atlassian API tokens do not expire automatically, but they can be revoked from the Atlassian account settings. If requests return 401:

1. Generate a new token at <https://id.atlassian.com/manage-profile/security/api-tokens>
2. Re-run `atla auth login` with the new token

---

## See also

- [Getting Started](./getting-started.md) — installation and first-time setup
- [Configuration](./configuration.md) — config keys, aliases, and environment variables
