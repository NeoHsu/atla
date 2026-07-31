from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "cargo-chef-cook.py"


@unittest.skipIf(os.name == "nt", "fake executable helper is POSIX-only")
class CargoChefCookTests(unittest.TestCase):
    def test_cook_rehydrates_an_isolated_workspace(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            repo = base / "repo"
            repo.mkdir()
            manifest = repo / "Cargo.toml"
            manifest.write_text("source worktree\n", encoding="utf-8")
            (repo / ".mise-recipe.json").write_text("{}\n", encoding="utf-8")

            marker = base / "invocation.json"
            fake_cargo = base / "cargo"
            fake_cargo.write_text(
                f"#!{sys.executable}\n"
                "import json, os, pathlib, sys\n"
                "cwd = pathlib.Path.cwd()\n"
                "(cwd / 'Cargo.toml').write_text('chef skeleton\\n', encoding='utf-8')\n"
                "pathlib.Path(os.environ['CHEF_MARKER']).write_text(\n"
                "    json.dumps({'cwd': str(cwd), 'args': sys.argv[1:]}),\n"
                "    encoding='utf-8',\n"
                ")\n",
                encoding="utf-8",
            )
            fake_cargo.chmod(0o755)

            environment = os.environ.copy()
            environment.update({"CARGO": str(fake_cargo), "CHEF_MARKER": str(marker)})
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--repo-root", str(repo)],
                capture_output=True,
                check=False,
                env=environment,
                text=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(manifest.read_text(encoding="utf-8"), "source worktree\n")
            invocation = json.loads(marker.read_text(encoding="utf-8"))
            self.assertNotEqual(Path(invocation["cwd"]), repo)
            self.assertFalse(Path(invocation["cwd"]).exists())
            self.assertEqual(invocation["args"][:3], ["chef", "cook", "--workspace"])
            self.assertIn(str((repo / "target").resolve()), invocation["args"])

    def test_missing_recipe_fails_before_invoking_cargo(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--repo-root", str(repo)],
                capture_output=True,
                check=False,
                text=True,
            )

            self.assertEqual(result.returncode, 2)
            self.assertIn("run cargo chef prepare first", result.stderr)


if __name__ == "__main__":
    unittest.main()
