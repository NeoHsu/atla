#!/usr/bin/env python3
"""Add mandatory archive SHA-256 verification to cargo-dist's PowerShell installer."""

from __future__ import annotations

import sys
from pathlib import Path

MARKER = "  $wc.downloadFile($url, $dir_path)\n"
REPLACEMENT = """  $wc.downloadFile($url, $dir_path)

  # cargo-dist 0.31 verifies archives in the shell installer but not PowerShell.
  # Fetch the release sidecar from the same authenticated origin and fail closed.
  $checksum_path = "$dir_path.sha256"
  $wc.downloadFile("$url.sha256", $checksum_path)
  $checksum_text = (Get-Content -Raw -Path $checksum_path).Trim()
  $expected_checksum = ($checksum_text -split '\\s+')[0].ToLowerInvariant()
  $actual_checksum = (Get-FileHash -Algorithm SHA256 -Path $dir_path).Hash.ToLowerInvariant()
  if ($actual_checksum -ne $expected_checksum) {
    Remove-Item -Force -ErrorAction SilentlyContinue $dir_path
    throw "ERROR: SHA-256 checksum verification failed for $artifact_name"
  }
"""


def main() -> int:
    path = Path("target/distrib/atla-installer.ps1")
    source = path.read_text(encoding="utf-8")
    if source.count(MARKER) != 1:
        raise ValueError("expected exactly one cargo-dist archive download marker")
    hardened = source.replace(MARKER, REPLACEMENT)
    path.write_text(hardened, encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        sys.stderr.write(f"failed to harden PowerShell installer: {error}\n")
        raise SystemExit(1) from error
