#!/usr/bin/env python3
"""Verify exact version lockstep between the atla CLI and bundled agent skill."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

SEMVER = re.compile(
    r"^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$"
)
VERSIONED_SKILL_INSTALL_DOCS = (
    Path("README.md"),
    Path("docs/agent-reference.md"),
    Path("docs/getting-started.md"),
)
VERSIONED_CLI_INSTALL_DOCS = (Path("README.md"), Path("docs/getting-started.md"))
DOCTOR_FIXTURE = Path("docs/schemas/fixtures/doctor-v1.json")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="repository root (defaults to this script's parent repository)",
    )
    parser.add_argument(
        "--tag",
        help="release tag to verify in addition to checked-in versions (for example v0.6.0)",
    )
    return parser.parse_args()


def read_utf8(path: Path) -> str:
    try:
        with path.open("r", encoding="utf-8") as handle:
            return handle.read()
    except OSError as error:
        raise ValueError(f"failed to read {path}: {error}") from error


def load_json(path: Path) -> dict[str, Any]:
    try:
        document = json.loads(read_utf8(path))
    except json.JSONDecodeError as error:
        raise ValueError(f"failed to parse {path}: {error}") from error
    if not isinstance(document, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return document


def package_version(path: Path, display_path: Path) -> str:
    in_package = False
    for raw_line in read_utf8(path).splitlines():
        line = raw_line.strip()
        if line.startswith("["):
            in_package = line == "[package]"
            continue
        if in_package:
            match = re.fullmatch(r'version\s*=\s*"([^"]+)"(?:\s*#.*)?', line)
            if match is not None:
                return match.group(1)
    raise ValueError(f"{display_path} has no string package.version")


def locked_versions(path: Path, package_name: str) -> set[str]:
    versions: set[str] = set()
    blocks = re.split(r"(?m)^\s*\[\[package\]\]\s*$", read_utf8(path))
    for block in blocks[1:]:
        name = re.search(r'(?m)^name\s*=\s*"([^"]+)"\s*$', block)
        version = re.search(r'(?m)^version\s*=\s*"([^"]+)"\s*$', block)
        if name is not None and name.group(1) == package_name and version is not None:
            versions.add(version.group(1))
    return versions


def verify(repo: Path, release_tag: str | None) -> tuple[str, list[str]]:
    cli_path = Path("crates/atla-cli/Cargo.toml")
    core_path = Path("crates/atla-core/Cargo.toml")
    cli_version = package_version(repo / cli_path, cli_path)
    core_version = package_version(repo / core_path, core_path)
    manifest_path = repo / "skills/atla-cli/compatibility.json"
    manifest = load_json(manifest_path)
    skill_path = repo / "skills/atla-cli/SKILL.md"
    skill = read_utf8(skill_path)
    lock_path = repo / "Cargo.lock"
    doctor_fixture = load_json(repo / DOCTOR_FIXTURE)
    errors: list[str] = []

    if not SEMVER.fullmatch(cli_version):
        errors.append(f"CLI package version is not SemVer: {cli_version!r}")
    expected_tag = f"v{cli_version}"
    expected_versions: dict[str, Any] = {
        "atla-core package": core_version,
        "skillVersion": manifest.get("skillVersion"),
        "cliVersion": manifest.get("cliVersion"),
    }
    for label, version in expected_versions.items():
        if version != cli_version:
            errors.append(f"{label} {version!r} does not match CLI {cli_version!r}")

    if manifest.get("schemaVersion") != 1:
        errors.append("skill compatibility schemaVersion must be 1")
    if manifest.get("compatibility") != "exact":
        errors.append("skill compatibility policy must be `exact`")
    if manifest.get("releaseTag") != expected_tag:
        errors.append(
            f"skill releaseTag {manifest.get('releaseTag')!r} does not match {expected_tag!r}"
        )

    for package_name in ("atla", "atla-core"):
        versions = locked_versions(lock_path, package_name)
        if versions != {cli_version}:
            errors.append(
                f"Cargo.lock {package_name} versions {sorted(versions)!r} do not match {cli_version!r}"
            )

    fixture_compatibility = doctor_fixture.get("skillCompatibility")
    fixture_versions = {
        "doctor fixture cliVersion": doctor_fixture.get("cliVersion"),
        "doctor fixture skillCompatibility.cliVersion": (
            fixture_compatibility.get("cliVersion")
            if isinstance(fixture_compatibility, dict)
            else None
        ),
        "doctor fixture skillCompatibility.skillVersion": (
            fixture_compatibility.get("skillVersion")
            if isinstance(fixture_compatibility, dict)
            else None
        ),
        "doctor fixture skillCompatibility.targetVersion": (
            fixture_compatibility.get("targetVersion")
            if isinstance(fixture_compatibility, dict)
            else None
        ),
    }
    for label, version in fixture_versions.items():
        if version != cli_version:
            errors.append(f"{label} {version!r} does not match CLI {cli_version!r}")

    compatibility_match = re.search(
        r"^compatibility:\s*Requires atla CLI ([^ ]+) exactly$", skill, re.MULTILINE
    )
    if compatibility_match is None:
        errors.append(
            "SKILL.md frontmatter has no exact atla CLI compatibility declaration"
        )
    elif compatibility_match.group(1) != cli_version:
        errors.append(
            "SKILL.md compatibility version "
            f"{compatibility_match.group(1)!r} does not match {cli_version!r}"
        )

    gate = f"atla doctor --skill-version {cli_version} --output json"
    if gate not in skill:
        errors.append(f"SKILL.md execution gate is missing `{gate}`")
    fallback = (
        "cargo install --locked --git https://github.com/NeoHsu/atla "
        f"--tag v{cli_version} atla"
    )
    if fallback not in skill:
        errors.append(f"SKILL.md old-CLI remediation is missing `{fallback}`")

    skill_install_source = f"https://github.com/NeoHsu/atla/tree/{expected_tag}"
    for relative_path in VERSIONED_SKILL_INSTALL_DOCS:
        document = read_utf8(repo / relative_path)
        if skill_install_source not in document:
            errors.append(
                f"{relative_path} is missing tag-pinned skill source {skill_install_source}"
            )

    cli_install_command = (
        "cargo install --locked --git https://github.com/NeoHsu/atla "
        f"--tag {expected_tag} atla"
    )
    for relative_path in VERSIONED_CLI_INSTALL_DOCS:
        if cli_install_command not in read_utf8(repo / relative_path):
            errors.append(
                f"{relative_path} is missing tag-pinned CLI install `{cli_install_command}`"
            )

    if release_tag is not None and release_tag != expected_tag:
        errors.append(f"release tag {release_tag!r} does not match {expected_tag!r}")

    return cli_version, errors


def main() -> int:
    args = parse_args()
    try:
        repo = args.repo.resolve(strict=True)
        version, errors = verify(repo, args.tag)
    except (OSError, ValueError) as error:
        sys.stderr.write(f"skill version check failed: {error}\n")
        return 1

    if errors:
        for error in errors:
            sys.stderr.write(f"skill version check failed: {error}\n")
        return 1

    sys.stdout.write(f"verified atla CLI and skill exact lockstep at {version}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
