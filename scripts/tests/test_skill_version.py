from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "check-skill-version.py"
FIXTURE_PATHS = (
    Path("Cargo.lock"),
    Path("README.md"),
    Path("crates/atla-cli/Cargo.toml"),
    Path("crates/atla-core/Cargo.toml"),
    Path("docs/agent-reference.md"),
    Path("docs/getting-started.md"),
    Path("docs/schemas/fixtures/doctor-v1.json"),
    Path("skills/atla-cli/SKILL.md"),
    Path("skills/atla-cli/compatibility.json"),
)


class SkillVersionTests(unittest.TestCase):
    def run_check(
        self, repo: Path, *arguments: str, expected: int = 0
    ) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--repo", str(repo), *arguments],
            capture_output=True,
            check=False,
            text=True,
        )
        self.assertEqual(result.returncode, expected, result.stderr)
        return result

    def copy_fixture(self, destination: Path) -> None:
        for relative_path in FIXTURE_PATHS:
            target = destination / relative_path
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative_path, target)

    def test_current_tree_is_in_exact_lockstep(self) -> None:
        result = self.run_check(ROOT, "--tag", "v0.6.0")
        self.assertIn("exact lockstep at 0.6.0", result.stdout)

    def test_mismatched_skill_version_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            self.copy_fixture(repo)
            manifest_path = repo / "skills/atla-cli/compatibility.json"
            try:
                manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError) as error:
                self.fail(f"failed to load fixture manifest: {error}")
            manifest["skillVersion"] = "0.5.1"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            result = self.run_check(repo, expected=1)
            self.assertIn(
                "skillVersion '0.5.1' does not match CLI '0.6.0'", result.stderr
            )

    def test_mismatched_release_tag_fails(self) -> None:
        result = self.run_check(ROOT, "--tag", "v0.5.1", expected=1)
        self.assertIn("release tag 'v0.5.1' does not match 'v0.6.0'", result.stderr)


if __name__ == "__main__":
    unittest.main()
