from __future__ import annotations

from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
MISE_CONFIG = ROOT / "mise.toml"
CI_WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
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
    "security",
    "skill:version",
    "test",
    "test:cli",
    "test:core",
    "test:e2e",
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
        self.assertIn(f"cargo-deny@{tools['cargo:cargo-deny']}", workflow)
        self.assertIn(f"cargo-llvm-cov@{tools['cargo:cargo-llvm-cov']}", workflow)

    def test_pr_gate_keeps_security_and_msrv_checks(self) -> None:
        commands = self.config["tasks"]["check:pr"]["run"]
        for command in (
            "python scripts/check-skill-version.py",
            "cargo +1.91 check --workspace",
            "cargo audit",
            "cargo deny check",
            'python -m unittest discover -s scripts/tests -p "test_*.py"',
        ):
            self.assertIn(command, commands)


if __name__ == "__main__":
    unittest.main()
