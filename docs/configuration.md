---
title: Configuration
description: Config keys, aliases, environment variables, and multi-profile setup for atla.
---

# Configuration

atla stores configuration in a TOML file at `~/.config/atla/config.toml`. Use the `atla config` commands to read and write values, or edit the file directly.

---

## Config file location

| File | Default path | Environment override |
|------|-------------|---------------------|
| Configuration | `~/.config/atla/config.toml` | `ATLA_CONFIG` |
| Credentials | `~/.config/atla/credentials.toml` | `ATLA_CREDENTIALS` |

Path resolution order (highest priority first):

1. `ATLA_CONFIG` / `ATLA_CREDENTIALS` environment variable
2. `$XDG_CONFIG_HOME/atla/config.toml` (if `XDG_CONFIG_HOME` is set)
3. `~/.config/atla/config.toml` (default on Linux and macOS)
4. `%APPDATA%\atla\config.toml` (Windows only)

Override examples:

```bash
export ATLA_CONFIG=/path/to/config.toml
export ATLA_CREDENTIALS=/path/to/credentials.toml
```

---

## Config keys reference

| Key | Type | Description |
|-----|------|-------------|
| `default.profile` | string | Name of the active profile |
| `profiles.<name>.instance` | string | Atlassian instance URL |
| `profiles.<name>.email` | string | Account email address |
| `profiles.<name>.credential-store` | string | `keyring` or `file` |
| `profiles.<name>.default-project` | string | Default Jira project key |
| `profiles.<name>.default-space` | string | Default Confluence space key |
| `aliases.<name>` | string | Command alias (expanded before parsing) |

---

## Full config.toml example

```toml
[default]
profile = "work"

[profiles.work]
instance = "https://example.atlassian.net"
email = "you@example.com"
credential_store = "keyring"
default_project = "PROJ"
default_space = "DEV"

[profiles.personal]
instance = "https://personal.atlassian.net"
email = "me@personal.com"
credential_store = "file"
default_project = "SIDE"
default_space = "NOTES"

[aliases]
mine = "jira search 'assignee = currentUser() order by updated desc'"
sprint = "jira search 'sprint in openSprints() order by rank'"
recent-pages = "confluence page list --space DEV --limit 10"
```

---

## Command reference

### `atla config set`

Set a configuration value.

```
atla config set <key> <value>
```

Examples:

```bash
# Set the default profile
atla config set default.profile work

# Set a profile's default project
atla config set profiles.work.default-project PROJ

# Set a profile's default Confluence space
atla config set profiles.work.default-space DEV

# Create an alias
atla config set aliases.mine "jira search 'assignee = currentUser() order by updated desc'"
```

---

### `atla config get`

Read a single configuration value.

```
atla config get <key>
```

Examples:

```bash
atla config get default.profile
# work

atla config get profiles.work.instance
# https://example.atlassian.net

atla config get aliases.mine
# jira search 'assignee = currentUser() order by updated desc'
```

---

### `atla config list`

List all configuration values.

```
atla config list [--output <format>]
```

| Flag | Description |
|------|-------------|
| `--output`, `-o` | Output format: `table` (default), `json`, `csv`, `keys` |

Examples:

```bash
# Default table output
atla config list

# JSON for scripting
atla config list --output json

# Keys only (useful for scripting)
atla config list --output keys
```

---

## Aliases

Aliases expand to full atla commands before argument parsing. They let you create shortcuts for frequently used commands.

### How aliases work

1. You type `atla mine`
2. atla looks up `aliases.mine` in config
3. The alias value replaces the command: `atla jira search 'assignee = currentUser() order by updated desc'`
4. Normal argument parsing proceeds on the expanded command

### Expansion order

Alias expansion happens **once** at the top level before command parsing. Aliases cannot reference other aliases (no recursive expansion).

### Creating aliases

```bash
atla config set aliases.mine "jira search 'assignee = currentUser() order by updated desc'"
atla config set aliases.sprint "jira search 'sprint in openSprints() order by rank'"
atla config set aliases.recent-pages "confluence page list --space DEV --limit 10"
```

### Using aliases

```bash
# These are equivalent:
atla mine
atla jira search 'assignee = currentUser() order by updated desc'
```

You can append additional flags after an alias:

```bash
atla mine --output json
# Expands to: atla jira search 'assignee = currentUser() order by updated desc' --output json
```

---

## Environment variables

| Variable | Description |
|----------|-------------|
| `ATLA_CONFIG` | Override config file path |
| `ATLA_CREDENTIALS` | Override credentials file path |
| `ATLA_TOKEN` | API token (overrides stored credentials) |
| `ATLA_API_TOKEN` | API token alias (same as `ATLA_TOKEN`) |

Priority order for tokens:

1. `ATLA_TOKEN` / `ATLA_API_TOKEN` environment variable
2. Stored credential (keyring or file, per profile)

---

## Multiple environments / profiles

Use profiles to manage multiple Atlassian instances from a single CLI installation.

### Pattern: work + personal

```toml
[default]
profile = "work"

[profiles.work]
instance = "https://company.atlassian.net"
email = "you@company.com"
credential_store = "keyring"
default_project = "TEAM"
default_space = "ENG"

[profiles.personal]
instance = "https://personal.atlassian.net"
email = "me@personal.com"
credential_store = "file"
default_project = "SIDE"
default_space = "NOTES"
```

### Pattern: CI / headless

In CI environments, use environment variables and file storage:

```bash
export ATLA_TOKEN="$ATLASSIAN_API_TOKEN"
atla jira search "project = PROJ AND status = Done" --output json --no-input
```

Or configure with a custom config path:

```bash
export ATLA_CONFIG=/etc/atla/config.toml
export ATLA_CREDENTIALS=/etc/atla/credentials.toml
```

### Pattern: per-command override

Use `--profile` without changing the default:

```bash
# Default profile (work)
atla jira issue view TEAM-100

# One-off with different profile
atla jira issue view SIDE-5 --profile personal
```

---

## See also

- [Getting Started](./getting-started.md) — installation and first-time setup
- [Authentication](./authentication.md) — login, token storage, and troubleshooting
