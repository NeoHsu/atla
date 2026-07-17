# Security Policy

## Supported versions

Security fixes are applied to the latest released version of atla.

| Version | Supported |
| --- | --- |
| Latest release | Yes |
| Older releases | No |

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private
[security advisory form](https://github.com/NeoHsu/atla/security/advisories/new) and include:

- the affected atla version and platform;
- the command or component involved;
- reproduction steps or a proof of concept;
- the expected impact;
- any suggested mitigation.

Never include real Atlassian API tokens, OAuth credentials, private Jira data, or Confluence
content. Use redacted requests and synthetic test data.

The report will be acknowledged as soon as practical. Confirmed vulnerabilities are fixed on the
private advisory before coordinated disclosure and a patched release.

## Security defaults

- API tokens are stored in the operating-system keyring by default.
- File-backed credentials are plain text, atomically replaced, and mode `0600` on Unix.
- Basic-auth credentials are attached only to the configured API origin and are stripped on
  cross-origin redirects.
- Destructive commands refuse to run without `--yes`.
- Discovery agents can enforce `--read-only` and context budgets before network access.
- Automated contexts should pass `--no-input` to prevent prompts.
- Use `--dry-run` to inspect a mutation before executing it.
- Releases publish SHA-256 checksums, checksum-verifying shell/PowerShell installers, GitHub
  build-provenance attestations for local/global artifacts, and a CycloneDX 1.5 binary SBOM.
