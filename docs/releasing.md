---
title: Release Procedure
description: Versioning, cargo-dist artifacts, SBOM, provenance, and supply-chain checks.
---

# Release procedure

## Before tagging

1. Update versions and `CHANGELOG.md`.
2. Run the workspace test, fmt, Clippy, RustSec, MSRV, doc, and CLI-surface checks.
3. Run cargo-dist plan/local/global artifact smoke tests.
4. Run `python3 scripts/verify-release-artifacts.py`; it checks archive contents,
   sidecar/manifest hashes, CycloneDX 1.5, and SBOM component hashes.
5. For broad Confluence sandbox testing, use
   `scripts/atla-live-smoke.py`; its `finish` command must report complete,
   with every command leaf classified and no active or trashed temporary resources.
6. Confirm every `uses:` reference in `.github/workflows/release.yml` is a full commit SHA.
7. Confirm release-job permissions are scoped (`contents: write` only on `host`).

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
git tag -s v1.0.0 -m 'atla v1.0.0'
git push origin v1.0.0
```

Do not push a release tag from a dirty or unreviewed tree. After publishing, install from each
supported channel and run `atla --version`, `atla completion bash`, and a JSON dry-run smoke test.
