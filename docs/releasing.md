---
title: Release Procedure
description: Versioning, cargo-dist artifacts, SBOM, provenance, and supply-chain checks.
---

# Release procedure

## Before tagging

1. Run `python3 scripts/check-skill-version.py`. It must confirm exact lockstep across the CLI,
   atla-core, Cargo.lock, `skills/atla-cli/compatibility.json`, SKILL.md gate, and tag-pinned install
   docs. The tag-triggered workflow repeats this check with `--tag "$GITHUB_REF_NAME"`.
2. Rename the combined `[Unreleased]` section to `[0.6.0]` with the actual release date and restore
   an empty `[Unreleased]` section plus compare links.
3. Run the workspace test, fmt, Clippy, RustSec, MSRV, doc, and CLI-surface checks.
4. Run cargo-dist plan/local/global artifact smoke tests.
5. Run `python3 scripts/verify-release-artifacts.py`; it checks archive contents,
   sidecar/manifest hashes, CycloneDX 1.5, and SBOM component hashes.
6. For bounded Jira and Confluence sandbox testing, follow
   [Live Sandbox Smoke Testing](./live-smoke.md). The ledger's `finish` command
   must report complete, with every selected remote operation classified and no
   active or trashed temporary resources.
7. Confirm every `uses:` reference in `.github/workflows/release.yml` is a full commit SHA.
8. Confirm release-job permissions are scoped (`contents: write` only on `host`).

## Generated workflow policy

`release.yml` is based on cargo-dist 0.31.0 but intentionally differs from raw generated output:

- GitHub Actions are pinned to full commits.
- cargo-dist, rustup, and cargo-cyclonedx installer scripts are downloaded, checked against pinned
  SHA-256 values, and only then executed.
- untrusted GitHub expression values enter shell steps through environment variables.
- cargo-cyclonedx 0.5.9 is used instead of cargo-dist's older 0.5.5 template because 0.5.9
  understands Cargo.lock v4 and includes registry checksums.
- the SBOM describes the distributed `atla` binary, is renamed to `atla.cdx.xml`, and uses
  CycloneDX 1.5.

For this reason `dist-workspace.toml` contains `allow-dirty = ["ci"]`. Never regenerate and commit
`release.yml` without reapplying these controls and rerunning the workflow security scan.

## Artifact set

A release must contain platform archives/installers, installer `.sha256` sidecars, `sha256.sum`,
`atla.cdx.xml`, `atla.cdx.xml.sha256`, and the cargo-dist manifest. GitHub build-provenance attestations cover
local and global artifacts. Installers verify archive checksums before installation; users can
additionally verify the attestation with GitHub CLI.

## Tagging

Use a signed, SemVer-compatible tag only after the release commit is reviewed:

```bash
git tag -s v0.6.0 -m 'atla v0.6.0'
git push origin v0.6.0
```

Do not push a release tag from a dirty or unreviewed tree. After publishing, install from each
supported channel and run `atla --version`, `atla completion bash`, and a JSON dry-run smoke test.
Install the skill from the exact release tag, run
`atla doctor --skill-version 0.6.0 --output json`, and require
`skillCompatibility.compatible: true` before completing the release.
