from __future__ import annotations

import unittest
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parents[2]
MISE_CONFIG = ROOT / "mise.toml"
CI_WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
SPEC_REFRESH_WORKFLOW = ROOT / ".github" / "workflows" / "spec-refresh.yml"
RELEASE_WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"
EXPECTED_TASKS = {
    "audit",
    "check:fast",
    "check:pr",
    "contract:check",
    "contract:update",
    "coverage",
    "deny",
    "fmt",
    "lint",
    "msrv",
    "sccache:stats",
    "security",
    "security:secrets",
    "skill:version",
    "test",
    "test:cli",
    "test:core",
    "test:e2e",
    "test:nextest",
    "tooling:test",
}


class MiseTaskTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.config = tomllib.loads(MISE_CONFIG.read_text(encoding="utf-8"))

    def test_expected_development_tasks_are_registered(self) -> None:
        tasks = self.config["tasks"]
        self.assertFalse(EXPECTED_TASKS - tasks.keys())
        for dangerous_name in ("live-smoke", "publish", "release", "tag"):
            self.assertNotIn(dangerous_name, tasks)

    def test_ci_tool_versions_match_mise(self) -> None:
        tools = self.config["tools"]
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(f"sccache@{tools['sccache']}", workflow)
        self.assertIn(f"cargo-audit@{tools['cargo:cargo-audit']}", workflow)
        self.assertIn(f"cargo-deny@{tools['cargo:cargo-deny']}", workflow)
        self.assertIn(f"cargo-llvm-cov@{tools['cargo:cargo-llvm-cov']}", workflow)
        self.assertIn(f"cargo-nextest@{tools['cargo:cargo-nextest']}", workflow)
        self.assertIn(f'GITLEAKS_VERSION: "{tools["gitleaks"]}"', workflow)
        self.assertIn('CARGO_INCREMENTAL: "0"', workflow)
        self.assertIn("RUSTC_WRAPPER: sccache", workflow)

    def test_script_runtimes_match_mise(self) -> None:
        tools = self.config["tools"]
        self.assertEqual(tools["node"], "24")
        self.assertEqual(tools["python"], "3.12.13")

        node_pin = f'node-version: "{tools["node"]}"'
        python_pin = f'python-version: "{tools["python"]}"'
        for workflow_path in (CI_WORKFLOW, SPEC_REFRESH_WORKFLOW):
            workflow = workflow_path.read_text(encoding="utf-8")
            self.assertIn(node_pin, workflow)
            self.assertIn(python_pin, workflow)

        release_workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(python_pin, release_workflow)

    def test_ci_msrv_job_pins_the_toolchain_as_an_input(self) -> None:
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        msrv_job = workflow.split("\n  coverage:", 1)[0].split("\n  msrv:", 1)[1]
        self.assertIn("toolchain: 1.91.0", msrv_job)
        self.assertIn("cargo check --workspace", msrv_job)

    def test_pr_gate_keeps_security_and_msrv_checks(self) -> None:
        commands = self.config["tasks"]["check:pr"]["run"]
        for command in (
            "gitleaks dir --redact --no-banner .",
            "python scripts/check-skill-version.py",
            "cargo +1.91 check --workspace",
            "cargo nextest run --workspace --status-level all",
            "cargo test --doc -p atla-core -p atla-jira-api",
            "cargo audit",
            "cargo deny check",
            'python -m unittest discover -s scripts/tests -p "test_*.py"',
        ):
            self.assertIn(command, commands)

    def test_secret_scan_is_part_of_local_and_ci_security_gates(self) -> None:
        command = "gitleaks dir --redact --no-banner ."
        tasks = self.config["tasks"]
        self.assertEqual(tasks["security:secrets"]["run"], command)
        self.assertIn(command, tasks["security"]["run"])
        self.assertIn(command, tasks["check:pr"]["run"])
        self.assertIn(command, CI_WORKFLOW.read_text(encoding="utf-8"))
        self.assertTrue((ROOT / ".gitleaks.toml").is_file())

    def test_retired_tooling_does_not_return(self) -> None:
        tools = self.config["tools"]
        tasks = self.config["tasks"]
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        for retired_tool in ("java", "zig", "cargo:cargo-chef"):
            self.assertNotIn(retired_tool, tools)
        for retired_task in ("chef:cook", "chef:prepare"):
            self.assertNotIn(retired_task, tasks)
        self.assertNotIn("cargo-chef", workflow)
        self.assertNotIn("cargo chef", workflow)


if __name__ == "__main__":
    unittest.main()
