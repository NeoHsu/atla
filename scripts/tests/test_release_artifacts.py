from __future__ import annotations

import hashlib
import importlib.util
import io
import stat
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "verify-release-artifacts.py"

spec = importlib.util.spec_from_file_location("verify_release_artifacts", SCRIPT)
if spec is None or spec.loader is None:
    raise RuntimeError(f"could not load {SCRIPT}")
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


class ReleaseArtifactTests(unittest.TestCase):
    def write_sidecar(self, path: Path) -> None:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        path.with_name(path.name + ".sha256").write_text(
            f"{digest}  {path.name}\n", encoding="utf-8"
        )

    def write_tar(self, path: Path, members: dict[str, bytes]) -> None:
        with tarfile.open(path, mode="w:xz") as archive:
            for name, content in members.items():
                info = tarfile.TarInfo(f"atla/{name}")
                info.mode = 0o755 if name == "atla" else 0o644
                info.size = len(content)
                archive.addfile(info, io.BytesIO(content))

    def test_safe_names_reject_traversal(self) -> None:
        with self.assertRaises(ValueError):
            verify.safe_names(["atla/../escape"], Path("archive.tar.xz"))
        with self.assertRaises(ValueError):
            verify.safe_names(["/absolute/path"], Path("archive.tar.xz"))

    def test_valid_binary_archive_and_checksum_are_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "atla-x86_64-unknown-linux-gnu.tar.xz"
            self.write_tar(
                archive,
                {
                    "atla": b"#!/bin/sh\necho atla 0.0.0\n",
                    "README.md": b"readme",
                    "CHANGELOG.md": b"changelog",
                    "LICENSE": b"license",
                },
            )
            self.write_sidecar(archive)

            verify.verify_binary_archive(root.resolve(), archive)

    def test_checksum_manifest_accepts_valid_entries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            artifact = root / "atla.cdx.xml"
            artifact.write_bytes(b"sbom")
            digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
            (root / "sha256.sum").write_text(
                f"{digest}  {artifact.name}\n", encoding="utf-8"
            )

            verify.verify_checksum_manifest(root)

    def test_checksum_manifest_rejects_paths_outside_distribution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve() / "dist"
            root.mkdir()
            outside = root.parent / "outside.bin"
            outside.write_bytes(b"outside")
            digest = hashlib.sha256(outside.read_bytes()).hexdigest()
            (root / "sha256.sum").write_text(
                f"{digest}  ../outside.bin\n", encoding="utf-8"
            )

            with self.assertRaises(ValueError):
                verify.verify_checksum_manifest(root)

    def test_checksum_manifest_rejects_malformed_digest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            artifact = root / "artifact.bin"
            artifact.write_bytes(b"artifact")
            (root / "sha256.sum").write_text(
                f"not-a-digest  {artifact.name}\n", encoding="utf-8"
            )

            with self.assertRaises(ValueError):
                verify.verify_checksum_manifest(root)

    def test_installers_require_checksum_verification_and_sidecars(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            shell = root / "atla-installer.sh"
            powershell = root / "atla-installer.ps1"
            shell.write_text("verify_checksum archive sha256sum", encoding="utf-8")
            powershell.write_text(
                "Get-FileHash -Algorithm SHA256 $url.sha256", encoding="utf-8"
            )
            self.write_sidecar(shell)
            self.write_sidecar(powershell)

            verify.verify_installers(root)

    def test_installer_without_checksum_verification_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            shell = root / "atla-installer.sh"
            powershell = root / "atla-installer.ps1"
            shell.write_text("download archive", encoding="utf-8")
            powershell.write_text(
                "Get-FileHash -Algorithm SHA256 $url.sha256", encoding="utf-8"
            )
            self.write_sidecar(shell)
            self.write_sidecar(powershell)

            with self.assertRaises(ValueError):
                verify.verify_installers(root)

    def test_native_smoke_executes_matching_archive_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "atla-x86_64-unknown-linux-gnu.tar.xz"
            self.write_tar(
                archive,
                {
                    "atla": b"#!/bin/sh\nprintf 'atla smoke\\n'\n",
                    "README.md": b"readme",
                    "CHANGELOG.md": b"changelog",
                    "LICENSE": b"license",
                },
            )
            with patch.object(
                verify, "native_target", return_value="x86_64-unknown-linux-gnu"
            ):
                verify.execute_native_archive(root.resolve(), [archive])

    def test_zip_symlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "atla-x86_64-pc-windows-msvc.zip"
            link = zipfile.ZipInfo("atla/atla.exe")
            link.external_attr = (stat.S_IFLNK | 0o777) << 16
            with zipfile.ZipFile(archive, mode="w") as package:
                package.writestr(link, b"target")

            with self.assertRaises(ValueError):
                verify.verify_binary_archive(root.resolve(), archive)

    def test_tar_hardlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "atla-x86_64-unknown-linux-gnu.tar.xz"
            with tarfile.open(archive, mode="w:xz") as package:
                link = tarfile.TarInfo("atla/atla")
                link.type = tarfile.LNKTYPE
                link.linkname = "target"
                package.addfile(link)

            with self.assertRaises(ValueError):
                verify.verify_binary_archive(root.resolve(), archive)


if __name__ == "__main__":
    unittest.main()
