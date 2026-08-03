from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "coverage-report.py"

spec = importlib.util.spec_from_file_location("coverage_report", SCRIPT)
if spec is None or spec.loader is None:
    raise RuntimeError(f"could not load {SCRIPT}")
coverage_report = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = coverage_report
spec.loader.exec_module(coverage_report)


class CoverageReportTests(unittest.TestCase):
    def test_collects_workspace_and_crate_totals(self) -> None:
        document = {
            "data": [
                {
                    "files": [
                        {
                            "filename": str(ROOT / "crates/atla-cli/src/main.rs"),
                            "summary": {"lines": {"covered": 8, "count": 10}},
                        },
                        {
                            "filename": str(ROOT / "crates/atla-core/src/lib.rs"),
                            "summary": {"lines": {"covered": 3, "count": 5}},
                        },
                    ]
                }
            ]
        }

        total, crates, files = coverage_report.collect_coverage(document)

        self.assertEqual(total, coverage_report.Coverage(11, 15))
        self.assertEqual(crates["atla-cli"], coverage_report.Coverage(8, 10))
        self.assertEqual(crates["atla-core"], coverage_report.Coverage(3, 5))
        self.assertEqual(len(files), 2)

    def test_ignores_malformed_and_empty_line_summaries(self) -> None:
        document = {
            "data": [
                {
                    "files": [
                        {
                            "filename": "crates/atla-cli/src/main.rs",
                            "summary": {"lines": {"covered": "bad", "count": 10}},
                        },
                        {
                            "filename": "crates/atla-cli/src/empty.rs",
                            "summary": {"lines": {"covered": 0, "count": 0}},
                        },
                    ]
                }
            ]
        }

        total, crates, files = coverage_report.collect_coverage(document)

        self.assertEqual(total, coverage_report.Coverage(0, 0))
        self.assertEqual(crates, {})
        self.assertEqual(files, [])

    def test_render_orders_lowest_coverage_first(self) -> None:
        summary = coverage_report.render_summary(
            coverage_report.Coverage(15, 20),
            {"atla": coverage_report.Coverage(15, 20)},
            [
                ("crates/atla-cli/src/high.rs", coverage_report.Coverage(9, 10)),
                ("crates/atla-cli/src/low.rs", coverage_report.Coverage(1, 10)),
            ],
            top=1,
        )

        self.assertIn("Workspace line coverage: **75.00%**", summary)
        self.assertIn("`crates/atla-cli/src/low.rs`", summary)
        self.assertNotIn("`crates/atla-cli/src/high.rs`", summary)

    def test_invalid_coverage_values_are_ignored(self) -> None:
        self.assertIsNone(
            coverage_report.parse_line_coverage({"covered": 2, "count": 1})
        )
        self.assertIsNone(
            coverage_report.parse_line_coverage({"covered": -1, "count": 1})
        )


if __name__ == "__main__":
    unittest.main()
