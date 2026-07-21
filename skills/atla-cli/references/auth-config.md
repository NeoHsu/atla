# Authentication & Configuration Reference

Use `atla auth --help` or `atla config --help` as the runtime syntax authority. Global execution
controls and safety gates live in `../SKILL.md`.

## Authentication

Token expiry is configured by Atlassian outside atla. Record the expiration shown when creating a
token and rotate it before that date; atla reports token availability/source, not expiry. Unscoped
tokens use the site URL. For scoped tokens, pass the tenant cloud ID;
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

For a scoped token, discover the tenant and add the returned Cloud ID to the non-interactive login
command above as `--cloud-id 11111111-2222-3333-4444-555555555555`:

```bash
atla --output json auth discover --site https://example.atlassian.net
```

For headless or CI storage, add `--storage file` to the same canonical login command. Do not replace
`--token-stdin` with a token-bearing process argument.

Instance URL is auto-normalized: `example.atlassian.net` becomes `https://example.atlassian.net`.

### Status

```bash
atla auth status
```

With `--output json`, `configured` is false and profile fields are null when no active profile
exists. Otherwise status shows the active profile, instance, email, credential storage, API
target/cloud ID, policy mode, and token availability/source.

### Diagnostics and local discovery

```bash
atla doctor --output json
atla doctor --network --timeout 10 --output json
atla --profile agent explain-policy jira.issue.create --output json
atla operation list --output json
atla schema list --output json
atla schema print error-v1 --output json
```

`doctor` checks config/profile/token-source/API-target/policy locally. `--network` additionally calls
the unauthenticated tenant-info endpoint to verify site reachability and discover the cloud ID; it
never prints token contents. `explain-policy` shows deny → allow → mode evaluation plus global
`--read-only`. The operation and schema commands do not need credentials or network access.

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

Login writes the profile selected by global `--profile`; without it, the profile name is `default`.
It does not derive a profile name from the instance/email pair. Create a named profile by adding
`--profile work` to the canonical login command, then switch to it:

```bash
atla auth switch work
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
| File | `--storage file` | Unix: `~/.config/atla/credentials.toml`; Windows: platform config directory | Headless, CI, containers |
| Environment | N/A | `ATLA_TOKEN` env var | CI pipelines, one-off runs |

Token precedence: `ATLA_TOKEN` > `ATLA_API_TOKEN` > stored credential (keyring/file).

---

## Configuration

Config file: Unix defaults to `~/.config/atla/config.toml`; Windows uses the platform config
directory. Run `atla doctor --output json` to see the resolved path.

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

# POSIX shell examples
atla completion bash > ~/.local/share/bash-completion/completions/atla
atla completion zsh > ~/.zsh/completions/_atla
atla completion fish > ~/.config/fish/completions/atla.fish
```

```powershell
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

Aliases expand before argument parsing and may chain through other aliases. Expansion stops after
at most eight steps; cycles fail instead of recursing indefinitely.

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
| `ATLA_CONFIG` | Config file path | Unix: `~/.config/atla/config.toml`; Windows: platform config directory |
| `ATLA_CREDENTIALS` | Credentials file path | Unix: `~/.config/atla/credentials.toml`; Windows: platform config directory |
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
| Policy unexpectedly blocks an operation | Deny/allow/mode or global read-only | Run `atla explain-policy jira.issue.create --output json` with the intended profile |
| Config, keyring, or site diagnosis unclear | Multiple possible setup failures | Run `atla doctor --output json`, then opt into `--network` if needed |
| 401 Unauthorized | Token expired or revoked | Generate a replacement token at id.atlassian.com, then re-login |
