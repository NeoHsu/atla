# Authentication & Configuration Reference

## Authentication

Atlassian API tokens expire after a configurable 1–365 days. Rotate stored tokens before
their expiry. Unscoped tokens use the site URL. For scoped tokens, pass the tenant cloud ID;
atla routes Jira and Confluence through their product-specific
`api.atlassian.com/ex/{product}/{cloudId}` gateways.

### Login

```
atla auth login [--instance URL] [--cloud-id ID] [--email EMAIL] [--token TOKEN | --token-stdin] [--storage keyring|file]
atla auth discover --site URL
```

Interactive (prompts for all values):

```bash
atla auth login
```

Non-interactive:

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --no-input \
  --instance https://example.atlassian.net --email you@example.com --token-stdin
```

Scoped token:

```bash
atla --output json auth discover --site https://example.atlassian.net
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --no-input \
  --instance https://example.atlassian.net \
  --cloud-id 11111111-2222-3333-4444-555555555555 \
  --email you@example.com --token-stdin
```

Headless/CI (file-backed):

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --no-input --storage file \
  --instance https://example.atlassian.net --email you@example.com --token-stdin
```

Instance URL is auto-normalized: `example.atlassian.net` becomes `https://example.atlassian.net`.

### Status

```bash
atla auth status
```

Shows: active profile, instance, email, credential storage, API target/cloud ID, policy mode, and token availability.

### Logout

```bash
atla auth logout --yes
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
| `schema-version` | Read current config schema (`2`) | `atla config get schema-version` |
| `default.profile` | Active profile name | `atla config set default.profile work` |
| `default-project` | Default Jira project (active profile) | `atla config set default-project PROJ` |
| `default-space` | Default Confluence space (active profile) | `atla config set default-space ENG` |
| `cloud-id` | Cloud ID for active profile; empty clears it | `atla config set cloud-id 11111111-2222-3333-4444-555555555555` |
| `profiles.<name>.instance` | Instance URL for a profile | `atla config set profiles.work.instance https://...` |
| `profiles.<name>.email` | Email for a profile | `atla config set profiles.work.email you@...` |
| `profiles.<name>.credential-store` | Storage backend | `atla config set profiles.work.credential-store file` |
| `profiles.<name>.cloud-id` | Cloud ID for scoped-token routing; empty clears it | `atla config set profiles.work.cloud-id 11111111-2222-3333-4444-555555555555` |
| `profiles.<name>.policy.mode` | `read-only` or `read-write` default | `atla config set profiles.agent.policy.mode read-only` |
| `profiles.<name>.policy.allow` | Comma-separated allowed operation patterns | `atla config set profiles.agent.policy.allow jira.issue.view,jira.issue.comment.add` |
| `profiles.<name>.policy.deny` | Comma-separated denied patterns (highest priority) | `atla config set profiles.agent.policy.deny '*.delete'` |
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
| `ATLA_READ_ONLY` | Enforce mutation blocking | unset/false |
| `XDG_CONFIG_HOME` | XDG base config directory (Linux & macOS) | `~/.config` |

---

Legacy unversioned configs are backed up as `config.toml.v1.bak` before migration. Config
and file-credential writes are atomic; Unix file mode is `0600`.

## Example config.toml

```toml
schema_version = 2

[default]
profile = "work"

[profiles.work]
instance = "https://example.atlassian.net"
email = "you@example.com"
credential_store = "keyring"
cloud_id = "11111111-2222-3333-4444-555555555555"
default_project = "PROJ"
default_space = "DEV"

[profiles.work.policy]
mode = "read-only"
deny = ["*.delete"]

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
| 401 Unauthorized | Token expired or revoked | Generate a replacement token at id.atlassian.com, then re-login |
