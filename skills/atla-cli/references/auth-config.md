# Authentication & Configuration Reference

## Authentication

### Login
```
atla auth login [--instance URL] [--email EMAIL] [--token TOKEN] [--storage keyring|file]
```

Interactive (prompts for all values):
```bash
atla auth login
```

Non-interactive:
```bash
atla auth login --instance https://example.atlassian.net \
  --email you@example.com --token "$ATLASSIAN_TOKEN"
```

Headless/CI (file-backed):
```bash
atla auth login --storage file --instance https://example.atlassian.net \
  --email you@example.com --token "$ATLASSIAN_TOKEN"
```

Instance URL is auto-normalized: `example.atlassian.net` becomes `https://example.atlassian.net`.

### Status
```bash
atla auth status
```
Shows: active profile, instance, email, credential storage, token availability.

### Logout
```bash
atla auth logout
```

### Switch profile
```bash
atla auth switch <profile>
```

---

## Multiple Profiles

Each unique instance/email pair creates a separate profile. Switch between them:

```bash
atla auth switch work
atla auth switch personal
```

Per-command override without switching:
```bash
atla --profile personal jira search "project = SIDE"
```

---

## Token Storage

| Method | Flag | Where stored | Best for |
|--------|------|-------------|----------|
| Keyring (default) | `--storage keyring` | OS credential manager | Developer workstations |
| File | `--storage file` | `~/.config/atla/credentials.toml` | Headless, CI, containers |
| Environment | N/A | `ATLA_TOKEN` env var | CI pipelines, one-off runs |

Token precedence: `ATLA_TOKEN` > `ATLA_API_TOKEN` > stored credential (keyring/file).

---

## Configuration

Config file: `~/.config/atla/config.toml`

### Commands
```bash
atla config set <key> <value>
atla config get <key>
atla config list [--output json|table|csv|keys]
```

### Shell Completion
```bash
# Generate completion script for supported shells
atla completion <shell>

# Examples
atla completion bash > ~/.local/share/bash-completion/completions/atla
atla completion zsh > ~/.zsh/completions/_atla
atla completion fish > ~/.config/fish/completions/atla.fish
atla completion powershell | Out-File -Encoding utf8 atla-completion.ps1
```

### Key Reference

| Key | Description | Example |
|-----|-------------|---------|
| `default.profile` | Active profile name | `atla config set default.profile work` |
| `default-project` | Default Jira project (active profile) | `atla config set default-project PROJ` |
| `default-space` | Default Confluence space (active profile) | `atla config set default-space ENG` |
| `profiles.<name>.instance` | Instance URL for a profile | `atla config set profiles.work.instance https://...` |
| `profiles.<name>.email` | Email for a profile | `atla config set profiles.work.email you@...` |
| `profiles.<name>.credential-store` | Storage backend | `atla config set profiles.work.credential-store file` |
| `profiles.<name>.default-project` | Default Jira project for profile | `atla config set profiles.work.default-project PROJ` |
| `profiles.<name>.default-space` | Default Confluence space for profile | `atla config set profiles.work.default-space ENG` |
| `aliases.<name>` | Command alias | `atla config set aliases.mine "jira search '...'"` |

---

## Aliases

Aliases expand before argument parsing (one level, no recursion).

```bash
atla config set aliases.mine "jira search 'assignee = currentUser() order by updated desc'"
atla config set aliases.sprint "jira search 'sprint in openSprints() order by rank'"
atla config set aliases.recent-pages "confluence page list --space DEV --limit 10"
```

Usage (extra flags append after expansion):
```bash
atla mine --output json --limit 25
```

---

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `ATLA_TOKEN` | API token override (highest priority) | unset |
| `ATLA_API_TOKEN` | Alternative token variable | unset |
| `ATLA_CONFIG` | Config file path | `~/.config/atla/config.toml` |
| `ATLA_CREDENTIALS` | Credentials file path | `~/.config/atla/credentials.toml` |
| `XDG_CONFIG_HOME` | XDG base config directory (Linux & macOS) | `~/.config` |

---

## Example config.toml

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

## Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| `No token available` | Profile exists but no credential stored | Re-run `atla auth login` |
| `Failed to access system keyring` | Headless/no keyring daemon | Use `--storage file` or `ATLA_TOKEN` |
| Wrong instance hit | Wrong profile active | `atla auth status` then `atla auth switch <name>` |
| 401 Unauthorized | Token revoked | Generate new token at id.atlassian.com, re-login |
