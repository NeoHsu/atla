from __future__ import annotations

import unittest
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parents[2]
MISE_CONFIG = ROOT / "mise.toml"
CI_WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
EXPECTED_TASKS = {
    "audit",
    "check:fast",
    "check:pr",
    "chef:cook",
    "chef:prepare",
    "contract:check",
    "contract:update",
    "coverage",
    "deny",
    "fmt",
    "lint",
    "msrv",
    "sccache:stats",
    "security",
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
        self.assertIn(f"cargo-deny@{tools['cargo:cargo-deny']}", workflow)
        self.assertIn(f"cargo-llvm-cov@{tools['cargo:cargo-llvm-cov']}", workflow)
        self.assertIn(f"cargo-chef@{tools['cargo:cargo-chef']}", workflow)
        self.assertIn(f"cargo-nextest@{tools['cargo:cargo-nextest']}", workflow)
        self.assertIn('CARGO_INCREMENTAL: "0"', workflow)
        self.assertIn("RUSTC_WRAPPER: sccache", workflow)

    def test_spec_filter_runtime_is_pinned(self) -> None:
        self.assertEqual(self.config["tools"]["node"], "24")

    def test_ci_msrv_job_pins_the_toolchain_as_an_input(self) -> None:
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        msrv_job = workflow.split("\n  coverage:", 1)[0].split("\n  msrv:", 1)[1]
        self.assertIn("toolchain: 1.91.0", msrv_job)
        self.assertIn("cargo check --workspace", msrv_job)

    def test_pr_gate_keeps_security_and_msrv_checks(self) -> None:
        commands = self.config["tasks"]["check:pr"]["run"]
        for command in (
            "python scripts/check-skill-version.py",
            "cargo +1.91 check --workspace",
            "cargo chef prepare --recipe-path .mise-recipe.json",
            "python scripts/cargo-chef-cook.py --recipe-path .mise-recipe.json",
            "cargo nextest run --workspace --status-level all",
            "cargo test --doc -p atla-core -p atla-jira-api",
            "cargo audit",
            "cargo deny check",
            'python -m unittest discover -s scripts/tests -p "test_*.py"',
        ):
            self.assertIn(command, commands)

    def test_cargo_chef_cook_never_runs_in_the_source_worktree(self) -> None:
        commands = self.config["tasks"]["check:pr"]["run"]
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertFalse(
            any(command.startswith("cargo chef cook") for command in commands)
        )
        self.assertNotIn("- run: cargo chef cook", workflow)
        self.assertIn("python3 scripts/cargo-chef-cook.py", workflow)


if __name__ == "__main__":
    unittest.main()
