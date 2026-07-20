from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

SCRIPT = Path(__file__).resolve().parents[1] / "spec-diff-summary.py"
SPEC_PATHS = (
    Path("specs/jira-v3-partial.json"),
    Path("specs/confluence-v2-partial.json"),
    Path("specs/confluence-v1-partial.json"),
)


def spec(operation_id: str, schema_name: str) -> dict[str, Any]:
    return {
        "openapi": "3.0.0",
        "info": {"title": "fixture", "version": "1"},
        "paths": {
            "/items": {
                "get": {
                    "operationId": operation_id,
                    "responses": {"200": {"description": "ok"}},
                }
            }
        },
        "components": {"schemas": {schema_name: {"type": "object"}}},
    }


class SpecDiffSummaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        subprocess.run(
            ["git", "-C", str(self.root), "config", "user.email", "test@example.com"],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(self.root), "config", "user.name", "Test"],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(self.root), "config", "commit.gpgsign", "false"],
            check=True,
        )
        for index, path in enumerate(SPEC_PATHS):
            destination = self.root / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(json.dumps(spec(f"oldOperation{index}", "OldSchema")))
        subprocess.run(["git", "-C", str(self.root), "add", "specs"], check=True)
        subprocess.run(
            ["git", "-C", str(self.root), "commit", "-qm", "fixture base"],
            check=True,
        )

    def tearDown(self) -> None:
        self.directory.cleanup()

    def run_script(
        self, *arguments: str, expected: int = 0
    ) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--repo", str(self.root), *arguments],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, expected, result.stderr)
        return result

    def test_reports_operation_schema_size_and_hash_changes(self) -> None:
        changed = spec("newOperation", "NewSchema")
        changed["paths"]["/created"] = {
            "post": {
                "operationId": "createItem",
                "responses": {"201": {"description": "created"}},
            }
        }
        (self.root / SPEC_PATHS[0]).write_text(json.dumps(changed, indent=2) + "\n")

        result = self.run_script()

        self.assertIn("`specs/jira-v3-partial.json`", result.stdout)
        self.assertIn("1 → 2 (+1)", result.stdout)
        self.assertIn("`newOperation`", result.stdout)
        self.assertIn("`createItem`", result.stdout)
        self.assertIn("`oldOperation0`", result.stdout)
        self.assertIn("New SHA-256", result.stdout)
        self.assertIn("Reviewer checklist", result.stdout)

    def test_writes_requested_output_file(self) -> None:
        output = self.root / "summary.md"

        result = self.run_script("--output", str(output))

        self.assertEqual(result.stdout, "")
        self.assertIn("No operation IDs changed.", output.read_text())

    def test_invalid_current_json_fails_closed(self) -> None:
        (self.root / SPEC_PATHS[0]).write_text("not json")

        result = self.run_script(expected=1)

        self.assertIn("invalid JSON", result.stderr)


if __name__ == "__main__":
    unittest.main()
