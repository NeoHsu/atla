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
    "deps:duplicates",
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
    "workflow:check",
    "workflow:security",
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
        self.assertIn(
            "mozilla-actions/sccache-action@fc920bf0ec8de6ee65d409111f7ec508035751ba",
            workflow,
        )
        self.assertIn(f'version: "v{tools["sccache"]}"', workflow)
        self.assertIn(f"cargo-audit@{tools['cargo:cargo-audit']}", workflow)
        self.assertIn(f"cargo-deny@{tools['cargo:cargo-deny']}", workflow)
        self.assertIn(f"cargo-llvm-cov@{tools['cargo:cargo-llvm-cov']}", workflow)
        self.assertIn(f"cargo-nextest@{tools['cargo:cargo-nextest']}", workflow)
        self.assertIn("github.com/rhysd/actionlint/cmd/actionlint@v1.7.12", workflow)
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
        self.assertIn("cargo check --workspace --all-targets --locked", msrv_job)

    def test_ci_cancels_obsolete_runs_and_bounds_jobs(self) -> None:
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("concurrency:", workflow)
        self.assertIn("cancel-in-progress: true", workflow)
        self.assertIn("workflow-security:", workflow)
        self.assertGreaterEqual(workflow.count("timeout-minutes:"), 5)

    def test_coverage_publishes_per_crate_summary(self) -> None:
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(
            "cargo llvm-cov report --locked --json --summary-only --output-path coverage-summary.json",
            workflow,
        )
        self.assertIn(
            "python3 scripts/coverage-report.py --input coverage-summary.json --output coverage-summary.md",
            workflow,
        )
        self.assertIn('cat coverage-summary.md >> "$GITHUB_STEP_SUMMARY"', workflow)
        coverage_task = self.config["tasks"]["coverage"]["run"]
        self.assertIn(
            "python scripts/coverage-report.py --input target/coverage-summary.json --output target/coverage-summary.md",
            coverage_task,
        )

    def test_release_runs_native_artifact_smoke(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(
            "python3 scripts/verify-release-artifacts.py --execute-native", workflow
        )
        self.assertIn("verify-platform-artifacts:", workflow)
        self.assertIn(
            "python scripts/verify-release-artifacts.py --platform-only --execute-native",
            workflow,
        )
        self.assertIn("pattern: artifacts-build-local-*", workflow)
        self.assertIn("needs.verify-platform-artifacts.result", workflow)

    def test_pr_gate_keeps_security_and_msrv_checks(self) -> None:
        commands = self.config["tasks"]["check:pr"]["run"]
        for command in (
            "gitleaks dir --redact --no-banner .",
            "python scripts/check-skill-version.py",
            "cargo +1.91 check --workspace --all-targets --locked",
            "cargo nextest run --workspace --locked --status-level all",
            "cargo test --doc --workspace --locked",
            "cargo audit",
            "cargo deny check",
            "scripts/check-workflows.sh",
            "zizmor --persona pedantic --min-severity medium --min-confidence medium .",
            'python -m unittest discover -s scripts/tests -p "test_*.py"',
        ):
            self.assertIn(command, commands)

    def test_workflow_security_tasks_use_pinned_tools(self) -> None:
        tools = self.config["tools"]
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(
            f"github.com/rhysd/actionlint/cmd/actionlint@v{tools['actionlint']}",
            workflow,
        )
        self.assertIn(f"version: {tools['zizmor']}", workflow)
        self.assertIn("zizmorcore/zizmor-action@", workflow)
        self.assertTrue((ROOT / "scripts/check-workflows.sh").stat().st_mode & 0o111)

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
