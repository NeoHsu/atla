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
        self.jira_baseline = self.root / "jira-baseline.json"
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
        self.jira_baseline.write_text(
            json.dumps(
                {
                    "schemaVersion": 1,
                    "id": "10001",
                    "key": "SANDBOX-1",
                    "fields": {"summary": "Protected issue"},
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
            "--group",
            "confluence-page-lifecycle",
            *extra,
        )

    def init_jira(self, *extra: str) -> None:
        self.run_script(
            "init",
            "--state",
            str(self.state),
            "--auth-status",
            str(self.auth),
            "--jira-baseline",
            str(self.jira_baseline),
            "--site",
            "https://example.atlassian.net/",
            "--profile",
            "work",
            "--target-issue",
            "SANDBOX-1",
            "--project-key",
            "SANDBOX",
            "--group",
            "jira-issue-lifecycle",
            *extra,
        )

    def test_coverage_registry_matches_the_cli_operation_registry(self) -> None:
        result = self.run_script("list-coverage")
        coverage = self.parse_json_value(result.stdout)
        self.assertIsInstance(coverage, list)
        script_operations = {item["operation"]: item["risk"] for item in coverage}
        source = self.read_text_file(OPERATION_SOURCE)
        source_operations = {
            operation: risk.lower()
            for operation, risk in re.findall(
                r'operation!\(\s*"([a-z.-]+)"\s*,.*?,\s*(Read|Write|Destructive)\s*,',
                source,
                re.DOTALL,
            )
            if operation == "auth.discover"
            or operation.startswith(("jira.", "confluence."))
        }
        self.assertEqual(script_operations, source_operations)
        self.assertEqual(
            {item["group"] for item in coverage},
            {
                "auth-discovery",
                "jira-issue-lifecycle",
                "jira-attachment-lifecycle",
                "confluence-page-lifecycle",
                "confluence-attachment-lifecycle",
                "cql-jql-search",
            },
        )

    def test_init_is_private_and_auto_skips_space_mutations(self) -> None:
        self.init()
        state = self.read_json_file(self.state)
        self.assertEqual(state["schemaVersion"], 2)
        self.assertEqual(state["preflight"]["confluenceBaseline"]["version"], 7)
        self.assertEqual(state["coverage"]["confluence.space.create"]["status"], "skip")
        self.assertEqual(state["coverage"]["jira.issue.view"]["status"], "skip")
        self.assertGreater(len(state["coverage"]), 70)
        if os.name != "nt":
            self.assertEqual(self.state.stat().st_mode & 0o777, 0o600)

    def test_unselected_group_cannot_be_recorded_as_executed(self) -> None:
        self.init()
        result = self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "jira.issue.view",
            "--result",
            "pass",
            "--evidence",
            "unexpected command output",
            expected=2,
        )
        self.assertIn("unselected group", result.stderr)

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
            "--group",
            "confluence-page-lifecycle",
            expected=2,
        )
        self.assertIn("not requested site", result.stderr)

    def test_jira_resources_generate_bounded_cleanup_commands(self) -> None:
        self.init_jira(
            "--group",
            "jira-attachment-lifecycle",
            "--allow-mutations",
            "--max-resources",
            "5",
            "--max-mutations",
            "5",
        )
        for operation, resource, parent in (
            ("jira.issue.create", "jira-issue:SANDBOX-2", None),
            (
                "jira.issue.attachment.upload",
                "jira-attachment:20001",
                ("jira-issue", "SANDBOX-2"),
            ),
            (
                "jira.issue.comment.add",
                "jira-comment:30001",
                ("jira-issue", "SANDBOX-2"),
            ),
            ("jira.issue.link.add", "jira-link:40001", None),
        ):
            arguments = [
                "record",
                "--state",
                str(self.state),
                "--operation",
                operation,
                "--result",
                "pass",
                "--evidence",
                "sandbox mutation output",
                "--resource",
                resource,
            ]
            if parent:
                arguments.extend(
                    ["--parent-type", parent[0], "--parent-id", parent[1]]
                )
            self.run_script(*arguments)

        cleanup = self.run_script("cleanup-commands", "--state", str(self.state))
        self.assertIn("jira issue delete SANDBOX-2 --yes", cleanup.stdout)
        self.assertIn("jira issue attachment delete 20001 --yes", cleanup.stdout)
        self.assertIn(
            "jira issue comment delete SANDBOX-2 30001 --yes", cleanup.stdout
        )
        self.assertIn("jira issue link remove 40001 --yes", cleanup.stdout)

    def test_mutation_and_resource_budgets_fail_closed(self) -> None:
        self.init(
            "--allow-mutations",
            "--max-mutations",
            "1",
            "--max-resources",
            "1",
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
        )
        result = self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "confluence.page.copy",
            "--result",
            "pass",
            "--evidence",
            "copy output",
            "--resource",
            "page:457",
            expected=2,
        )
        self.assertIn("mutation budget exhausted", result.stderr)
        result = self.run_script(
            "resource-add",
            "--state",
            str(self.state),
            "--type",
            "page",
            "--id",
            "458",
            "--evidence",
            "read-back output",
            expected=2,
        )
        self.assertIn("resource budget exhausted", result.stderr)

    def test_failures_require_drift_classification(self) -> None:
        self.init_jira()
        result = self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "jira.issue.view",
            "--result",
            "fail",
            "--evidence",
            "request failed",
            expected=2,
        )
        self.assertIn("require --failure-class", result.stderr)
        self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "jira.issue.view",
            "--result",
            "fail",
            "--failure-class",
            "api-drift",
            "--evidence",
            "request failed",
        )
        state = self.read_json_file(self.state)
        self.assertEqual(
            state["coverage"]["jira.issue.view"]["failureClass"], "api-drift"
        )

    def test_non_reversible_resources_require_documented_residue(self) -> None:
        self.init_jira("--allow-mutations", "--allow-residue")
        self.run_script(
            "record",
            "--state",
            str(self.state),
            "--operation",
            "jira.sprint.create",
            "--result",
            "pass",
            "--evidence",
            "created sandbox sprint",
            "--resource",
            "jira-sprint:77",
        )
        result = self.run_script(
            "resource-set",
            "--state",
            str(self.state),
            "--type",
            "jira-sprint",
            "--id",
            "77",
            "--to",
            "residue",
            expected=2,
        )
        self.assertIn("require --reason", result.stderr)
        self.run_script(
            "resource-set",
            "--state",
            str(self.state),
            "--type",
            "jira-sprint",
            "--id",
            "77",
            "--to",
            "residue",
            "--reason",
            "Jira exposes no sprint delete command; sandbox owner approved residue",
        )
        status = self.run_script(
            "status", "--state", str(self.state), "--json"
        )
        self.assertEqual(self.parse_json(status.stdout)["residueResources"], 1)


if __name__ == "__main__":
    unittest.main()
