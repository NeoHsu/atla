---
title: Getting Started
description: Install atla and set up your first Atlassian CLI workflow.
---

# Getting Started

**atla** is a unified command-line interface for Atlassian Jira and Confluence. It lets you search issues, manage boards, read and publish Confluence pages, and more — all from your terminal.

Key features:

- Profile-based authentication with multiple Atlassian instances
- Multiple output formats: `table`, `json`, `csv`, `keys`
- Shell completions for bash, zsh, fish, and PowerShell
- Command aliases for common workflows

---

## Installation

### Installer script (macOS / Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NeoHsu/atla/releases/latest/download/atla-installer.sh | sh
```

### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/NeoHsu/atla/releases/latest/download/atla-installer.ps1 | iex"
```

### mise

```bash
mise use -g github:NeoHsu/atla
```

### From source (requires Rust toolchain)

```bash
cargo install --git https://github.com/NeoHsu/atla atla
```

### Direct downloads

Pre-built binaries are available for:

| Platform | Architecture |
|----------|-------------|
| macOS | Apple Silicon (aarch64) |
| macOS | Intel (x86_64) |
| Linux | ARM64 (aarch64) |
| Linux | x64 (x86_64) |
| Windows | x64 (x86_64) |

Download from the [latest release](https://github.com/NeoHsu/atla/releases/latest).

---

## First-time setup

### 1. Log in to your Atlassian instance

```bash
atla auth login
```

The interactive prompt will ask for:

1. Your Atlassian instance URL (e.g. `https://example.atlassian.net`)
2. Your email address
3. An API token (generate one at <https://id.atlassian.com/manage-profile/security/api-tokens>)

Atlassian tokens expire after a configurable 1–365 days. Record the chosen expiry and rotate the
stored token before it expires. Unscoped tokens use your `https://SITE.atlassian.net` URL. For a
scoped token, pass `--cloud-id`; atla uses separate Jira and Confluence
`api.atlassian.com/ex/{product}/{cloudId}` gateway roots.

For non-interactive setup, pass the token over stdin so it is absent from shell history and
process arguments:

```bash
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --no-input \
  --instance https://example.atlassian.net \
  --email you@example.com \
  --token-stdin
```

For a scoped token:

```bash
atla --output json auth discover --site https://example.atlassian.net
printf '%s\n' "$ATLASSIAN_TOKEN" | atla auth login --no-input \
  --instance https://example.atlassian.net \
  --cloud-id 11111111-2222-3333-4444-555555555555 \
  --email you@example.com \
  --token-stdin
```

### 2. Verify your authentication

```bash
atla auth status
```

This confirms your active profile, instance, and credential storage method.

### 3. Set a default Jira project

```bash
atla config set profiles.work.default-project PROJ
```

### 4. Set a default Confluence space

```bash
atla config set profiles.work.default-space DEV
```

### 5. Install the bundled agent skill

If you use an AI coding assistant, install the bundled `atla-cli` skill so the agent knows
current `atla` commands, flags, pagination behavior, JQL/CQL patterns, and safety rules.

From GitHub:

```bash
npx skills add NeoHsu/atla --skill atla-cli
```

From a local checkout of this repo:

```bash
npx skills add . --skill atla-cli
```

For non-interactive setup across all supported agents:

```bash
npx skills add . --skill atla-cli --agent '*' -y
```

Use `--copy` if you want a standalone installed copy instead of a symlink back to the repo.

### 6. Create a useful alias

```bash
atla config set aliases.mine "jira search 'assignee = currentUser() order by updated desc'"
```

Now `atla mine` expands to your assigned issues.

---

## Shell completions

Set up tab completion for your shell:

### Bash

```bash
atla completion bash > ~/.local/share/bash-completion/completions/atla
```

### Zsh

```bash
atla completion zsh > ~/.zfunc/_atla
```

Add `fpath+=~/.zfunc` to your `.zshrc` before `compinit` if not already present.

### Fish

```bash
atla completion fish > ~/.config/fish/completions/atla.fish
```

### PowerShell

```powershell
atla completion powershell >> $PROFILE
```

---

## Quick demo

Once authenticated, try these commands:

### Search Jira issues assigned to you

```bash
atla jira search "assignee = currentUser() ORDER BY updated DESC"
```

### View a specific issue

```bash
atla jira issue view PROJ-123
```

### List Confluence spaces

```bash
atla confluence space list
```

### View a Confluence page as JSON

```bash
atla confluence page view 12345 --output json
```

### Use a different output format

```bash
atla jira search "project = PROJ" --output csv
```

---

## Global flags

These flags work with any command:

| Flag | Short | Description |
|------|-------|-------------|
| `--output` | `-o` | Output format: `table`, `json`, `csv`, `keys` |
| `--profile` | | Use a specific named profile |
| `--verbose` | | Enable verbose/debug output |
| `--dry-run` | | Show what would happen without making changes |
| `--read-only` | | Reject every local or remote mutation |
| `--max-pages` | | Stop automatic pagination after N API pages |
| `--max-items` | | Return at most N records |
| `--max-bytes` | | Refuse oversized structured output |
| `--timeout` | | Set a per-request timeout in seconds |
| `--no-input` | | Disable interactive prompts |

---

## Next steps

- [Authentication](./authentication.md) — manage profiles, tokens, and storage strategies
- [Configuration](./configuration.md) — config keys, aliases, and multi-environment setup
- [Agent Reference](./agent-reference.md) — skill installation and automation-focused command reference
