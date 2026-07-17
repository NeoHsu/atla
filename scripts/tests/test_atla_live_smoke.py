from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

SCRIPT = Path(__file__).resolve().parents[1] / "atla-live-smoke.py"
OPERATION_SOURCE = SCRIPT.parents[1] / "crates" / "atla-cli" / "src" / "operation.rs"


class LiveSmokeLedgerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.auth = self.root / "auth.json"
        self.baseline = self.root / "baseline.json"
        self.state = self.root / "state.json"
        self.auth.write_text(
            json.dumps(
                {
                    "schemaVersion": 1,
                    "profile": "work",
                    "instance": "https://example.atlassian.net",
                    "policy_mode": "read-write",
                    "api_target": "site",
                }
            )
        )
        self.baseline.write_text(
            json.dumps(
                {
                    "schemaVersion": 1,
                    "id": "123",
                    "title": "CLI TEST",
                    "version": {"number": 7},
                }
            )
        )

    def tearDown(self) -> None:
        self.directory.cleanup()

    def run_script(
        self, *arguments: str, expected: int = 0
    ) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), *arguments],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, expected, result.stderr)
        return result

    def read_text_file(self, path: Path) -> str:
        try:
            return path.read_text()
        except OSError as error:
            self.fail(f"failed to read test file `{path}`: {error}")

    def read_json_file(self, path: Path) -> dict[str, Any]:
        try:
            value = json.loads(self.read_text_file(path))
        except json.JSONDecodeError as error:
            self.fail(f"failed to read test JSON `{path}`: {error}")
        if not isinstance(value, dict):
            self.fail(f"test JSON `{path}` is not an object")
        return value

    def parse_json_value(self, value: str) -> Any:
        try:
            return json.loads(value)
        except json.JSONDecodeError as error:
            self.fail(f"failed to parse command JSON: {error}")

    def parse_json(self, value: str) -> dict[str, Any]:
        parsed = self.parse_json_value(value)
        if not isinstance(parsed, dict):
            self.fail("command JSON is not an object")
        return parsed

    def init(self, *extra: str) -> None:
        self.run_script(
            "init",
            "--state",
            str(self.state),
            "--auth-status",
            str(self.auth),
            "--baseline",
            str(self.baseline),
            "--site",
            "https://example.atlassian.net/",
            "--profile",
            "work",
            "--target-page",
            "123",
            "--space-key",
            "ENG",
            *extra,
        )

    def test_coverage_registry_matches_the_cli_operation_registry(self) -> None:
        result = self.run_script("list-coverage")
        coverage = self.parse_json_value(result.stdout)
        self.assertIsInstance(coverage, list)
        script_operations = {item["operation"] for item in coverage}
        source_operations = set(
            re.findall(
                r'"(confluence\.[a-z.]+)"', self.read_text_file(OPERATION_SOURCE)
            )
        )
        self.assertEqual(script_operations, source_operations)

    def test_init_is_private_and_auto_skips_space_mutations(self) -> None:
        self.init()
        state = self.read_json_file(self.state)
        self.assertEqual(state["preflight"]["baseline"]["version"], 7)
        self.assertEqual(state["coverage"]["confluence.space.create"]["status"], "skip")
        self.assertEqual(len(state["coverage"]), 36)
        if os.name != "nt":
            self.assertEqual(self.state.stat().st_mode & 0o777, 0o600)

    def test_read_only_ledger_rejects_mutation_pass_and_finish_is_fail_closed(
        self,
    ) -> None:
        self.init()
        self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "confluence.page.create",
            "--result",
            "pass",
            "--evidence",
            "created page",
            expected=2,
        )
        result = self.run_script("finish", "--state", str(self.state), expected=2)
        self.assertIn("still pending", result.stderr)

    def test_fully_classified_read_only_ledger_can_finish(self) -> None:
        self.init()
        state = self.read_json_file(self.state)
        for operation, entry in state["coverage"].items():
            if entry["status"] == "skip":
                continue
            if entry["risk"] == "read":
                self.run_script(
                    "record",
                    "--state",
                    str(self.state),
                    "--operation",
                    operation,
                    "--result",
                    "pass",
                    "--evidence",
                    "read-only command log",
                )
            else:
                self.run_script(
                    "record",
                    "--state",
                    str(self.state),
                    "--operation",
                    operation,
                    "--result",
                    "skip",
                    "--reason",
                    "mutations not authorized",
                )
        result = self.run_script("finish", "--state", str(self.state))
        self.assertEqual(self.parse_json(result.stdout)["status"], "complete")

    def test_resource_ledger_protects_target_and_generates_cleanup_commands(
        self,
    ) -> None:
        self.init("--allow-mutations", "--allow-purge")
        self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "confluence.page.create",
            "--result",
            "pass",
            "--evidence",
            "create output",
            expected=2,
        )
        self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "confluence.page.create",
            "--result",
            "pass",
            "--evidence",
            "create output",
            "--resource",
            "page:123",
            expected=2,
        )
        self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "confluence.page.create",
            "--result",
            "pass",
            "--evidence",
            "create output",
            "--resource",
            "page:456",
            "--parent-type",
            "page",
            "--parent-id",
            "123",
        )
        active = self.run_script("cleanup-commands", "--state", str(self.state))
        self.assertIn("page delete 456 --yes", active.stdout)
        self.run_script(
            "resource-set",
            "--state",
            str(self.state),
            "--type",
            "page",
            "--id",
            "456",
            "--to",
            "trashed",
        )
        trashed = self.run_script("cleanup-commands", "--state", str(self.state))
        self.assertIn("page delete 456 --purge --yes", trashed.stdout)
        self.run_script(
            "resource-set",
            "--state",
            str(self.state),
            "--type",
            "page",
            "--id",
            "456",
            "--to",
            "purged",
        )

    def test_init_rejects_profile_or_site_mismatch(self) -> None:
        result = self.run_script(
            "init",
            "--state",
            str(self.state),
            "--auth-status",
            str(self.auth),
            "--baseline",
            str(self.baseline),
            "--site",
            "https://other.atlassian.net",
            "--profile",
            "work",
            "--target-page",
            "123",
            "--space-key",
            "ENG",
            expected=2,
        )
        self.assertIn("not requested site", result.stderr)


if __name__ == "__main__":
    unittest.main()
