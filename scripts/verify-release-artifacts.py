#!/usr/bin/env python3
"""Verify cargo-dist archives, checksums, and the CycloneDX binary SBOM."""

from __future__ import annotations

import argparse
import hashlib
import platform
import stat
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path, PurePosixPath


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def verify_sidecar(path: Path) -> None:
    sidecar = path.with_name(path.name + ".sha256")
    if not sidecar.is_file():
        raise ValueError(f"missing checksum sidecar for {path.name}")
    expected = sidecar.read_text(encoding="utf-8").split()[0]
    actual = digest(path)
    if expected != actual:
        raise ValueError(f"checksum mismatch for {path.name}")


def safe_names(names: list[str], archive: Path) -> set[str]:
    basenames: set[str] = set()
    for name in names:
        normalized = PurePosixPath(name.replace("\\", "/"))
        if normalized.is_absolute() or ".." in normalized.parts:
            raise ValueError(f"unsafe path {name!r} in {archive.name}")
        if normalized.name:
            basenames.add(normalized.name)
    return basenames


def checked_zip_names(archive: zipfile.ZipFile, path: Path) -> set[str]:
    members = archive.infolist()
    for member in members:
        safe_names([member.filename], path)
        mode = member.external_attr >> 16
        if stat.S_IFMT(mode) == stat.S_IFLNK:
            raise ValueError(f"archive contains an unexpected symlink: {member.filename}")
    return safe_names([member.filename for member in members], path)


def checked_tar_names(archive: tarfile.TarFile, path: Path) -> set[str]:
    members = archive.getmembers()
    for member in members:
        safe_names([member.name], path)
        if member.issym() or member.islnk():
            raise ValueError(f"archive contains an unexpected link: {member.name}")
    return safe_names([member.name for member in members], path)


def verify_binary_archive(directory: Path, path: Path) -> None:
    path = constrained(directory, path)
    if path.suffix == ".zip":
        with zipfile.ZipFile(path) as archive:
            names = checked_zip_names(archive, path)
        executable = "atla.exe"
    else:
        with tarfile.open(path, mode="r:xz") as archive:
            names = checked_tar_names(archive, path)
        executable = "atla"
    required = {executable, "README.md", "CHANGELOG.md", "LICENSE"}
    missing = required - names
    if missing:
        raise ValueError(f"{path.name} is missing: {', '.join(sorted(missing))}")
    verify_sidecar(path)


def verify_source_archive(directory: Path, path: Path) -> None:
    path = constrained(directory, path)
    with tarfile.open(path, mode="r:gz") as archive:
        names = checked_tar_names(archive, path)
    required = {"Cargo.toml", "Cargo.lock", "README.md", "LICENSE"}
    missing = required - names
    if missing:
        raise ValueError(f"{path.name} is missing: {', '.join(sorted(missing))}")
    verify_sidecar(path)


def binary_from_archive(directory: Path, path: Path) -> tuple[bytes, int]:
    path = constrained(directory, path)
    executable = "atla.exe" if path.suffix == ".zip" else "atla"
    found: list[tuple[bytes, int]] = []
    if path.suffix == ".zip":
        with zipfile.ZipFile(path) as archive:
            checked_zip_names(archive, path)
            for member in archive.infolist():
                if not member.is_dir() and PurePosixPath(member.filename).name == executable:
                    found.append((archive.read(member), member.external_attr >> 16))
    else:
        with tarfile.open(path, mode="r:xz") as archive:
            checked_tar_names(archive, path)
            for member in archive.getmembers():
                if member.isfile() and PurePosixPath(member.name).name == executable:
                    source = archive.extractfile(member)
                    if source is None:
                        raise ValueError(f"could not read {executable} from {path.name}")
                    found.append((source.read(), member.mode))
    if len(found) != 1:
        raise ValueError(f"{path.name} should contain exactly one {executable}")
    return found[0]


def native_target() -> str | None:
    architecture = {
        "amd64": "x86_64",
        "x86_64": "x86_64",
        "arm64": "aarch64",
        "aarch64": "aarch64",
    }.get(platform.machine().lower())
    system = {
        "Darwin": "apple-darwin",
        "Linux": "unknown-linux-gnu",
        "Windows": "pc-windows-msvc",
    }.get(platform.system())
    return f"{architecture}-{system}" if architecture and system else None


def execute_native_archive(directory: Path, archives: list[Path]) -> None:
    target = native_target()
    if target is None:
        raise ValueError("cannot determine the native release target")
    archive = next((path for path in archives if target in path.name), None)
    if archive is None:
        raise ValueError(f"no release archive found for native target {target}")
    data, mode = binary_from_archive(directory, archive)
    with tempfile.TemporaryDirectory(prefix="atla-release-smoke-") as temp_dir:
        binary = Path(temp_dir) / ("atla.exe" if archive.suffix == ".zip" else "atla")
        binary.write_bytes(data)
        if archive.suffix != ".zip":
            binary.chmod(mode | stat.S_IXUSR)
        for arguments in (("--version",), ("--help",)):
            result = subprocess.run(
                [str(binary), *arguments],
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
            if "atla" not in f"{result.stdout}{result.stderr}".lower():
                raise ValueError(f"native archive returned unexpected output for {arguments[0]}")


def verify_sbom(path: Path) -> None:
    verify_sidecar(path)
    if path.stat().st_size > 16 * 1024 * 1024:
        raise ValueError("atla.cdx.xml exceeds the 16 MiB validation limit")
    document = path.read_text(encoding="utf-8")
    if 'xmlns="http://cyclonedx.org/schema/bom/1.5"' not in document:
        raise ValueError("atla.cdx.xml is not CycloneDX 1.5")
    if "<component " not in document or '<hash alg="SHA-256">' not in document:
        raise ValueError("SBOM has no components or SHA-256 component hashes")


def verify_installers(directory: Path) -> None:
    shell = constrained(directory, directory / "atla-installer.sh")
    powershell = constrained(directory, directory / "atla-installer.ps1")
    shell_text = shell.read_text(encoding="utf-8")
    powershell_text = powershell.read_text(encoding="utf-8")
    if "verify_checksum" not in shell_text or "sha256sum" not in shell_text:
        raise ValueError("shell installer does not verify archive checksums")
    if (
        "Get-FileHash -Algorithm SHA256" not in powershell_text
        or "$url.sha256" not in powershell_text
    ):
        raise ValueError("PowerShell installer does not verify archive checksums")
    verify_sidecar(shell)
    verify_sidecar(powershell)


def verify_checksum_manifest(directory: Path) -> None:
    manifest = directory / "sha256.sum"
    if not manifest.is_file():
        raise ValueError("missing sha256.sum")
    for line in manifest.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        expected, name = line.split(maxsplit=1)
        artifact = directory / name.lstrip(" *")
        if not artifact.is_file() or digest(artifact) != expected:
            raise ValueError(f"sha256.sum mismatch for {artifact.name}")


def constrained(directory: Path, path: Path) -> Path:
    resolved = path.resolve(strict=True)
    if not resolved.is_relative_to(directory):
        raise ValueError(f"artifact escapes distribution directory: {path}")
    return resolved


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--execute-native",
        action="store_true",
        help="run --version and --help from the archive matching the current runner",
    )
    arguments = parser.parse_args()
    directory = Path("target/distrib").resolve(strict=True)
    archives = [
        constrained(directory, path)
        for path in sorted(directory.glob("atla-*.tar.xz"))
        + sorted(directory.glob("atla-*.zip"))
    ]
    if not archives:
        raise ValueError("no atla platform archives found")
    for archive in archives:
        verify_binary_archive(directory, archive)
    if arguments.execute_native:
        execute_native_archive(directory, archives)
    verify_source_archive(directory, directory / "source.tar.gz")
    verify_sbom(constrained(directory, directory / "atla.cdx.xml"))
    verify_installers(directory)
    verify_checksum_manifest(directory)
    sys.stdout.write(
        f"verified {len(archives)} platform archive(s), source archive, checksums, and SBOM\n"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, tarfile.TarError, zipfile.BadZipFile) as error:
        sys.stderr.write(f"artifact verification failed: {error}\n")
        raise SystemExit(1) from error
