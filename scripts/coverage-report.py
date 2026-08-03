#!/usr/bin/env python3
"""Render a useful per-crate coverage summary from cargo-llvm-cov JSON."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class Coverage:
    covered: int
    total: int

    @property
    def percent(self) -> float:
        return 100.0 * self.covered / self.total if self.total else 100.0

    def combine(self, other: Coverage) -> Coverage:
        return Coverage(self.covered + other.covered, self.total + other.total)


def crate_name(filename: str) -> str | None:
    parts = Path(filename).parts
    try:
        crates_index = parts.index("crates")
        return parts[crates_index + 1]
    except (ValueError, IndexError):
        return None


def relative_filename(filename: str) -> str:
    path = Path(filename)
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def extract_files(document: dict[str, Any]) -> list[dict[str, Any]]:
    files: list[dict[str, Any]] = []
    data_sets = document.get("data", [])
    if not isinstance(data_sets, list):
        return files
    for data in data_sets:
        if not isinstance(data, dict) or not isinstance(data.get("files"), list):
            continue
        files.extend(file for file in data["files"] if isinstance(file, dict))
    return files


def parse_line_coverage(lines: dict[str, Any]) -> Coverage | None:
    try:
        covered = int(lines.get("covered", 0))
        total = int(lines.get("count", 0))
    except (TypeError, ValueError):
        return None
    if covered < 0 or total < 0 or covered > total:
        return None
    return Coverage(covered=covered, total=total)


def collect_coverage(
    document: dict[str, Any],
) -> tuple[Coverage, dict[str, Coverage], list[tuple[str, Coverage]]]:
    total = Coverage(0, 0)
    crates: dict[str, Coverage] = {}
    files: list[tuple[str, Coverage]] = []

    for file in extract_files(document):
        filename = file.get("filename")
        summary = file.get("summary")
        lines = summary.get("lines") if isinstance(summary, dict) else None
        if not isinstance(filename, str) or not isinstance(lines, dict):
            continue
        coverage = parse_line_coverage(lines)
        if coverage is None or coverage.total == 0:
            continue
        total = total.combine(coverage)
        name = crate_name(filename)
        if name is not None:
            crates[name] = crates.get(name, Coverage(0, 0)).combine(coverage)
        files.append((relative_filename(filename), coverage))

    return total, crates, files


def render_summary(
    total: Coverage,
    crates: dict[str, Coverage],
    files: list[tuple[str, Coverage]],
    top: int,
) -> str:
    lines = [
        "# Coverage summary",
        "",
        f"Workspace line coverage: **{total.percent:.2f}%** ({total.covered:,}/{total.total:,})",
        "",
        "## By crate",
        "",
        "| Crate | Covered lines | Total lines | Coverage |",
        "| --- | ---: | ---: | ---: |",
    ]
    for name, coverage in sorted(crates.items()):
        lines.append(
            f"| `{name}` | {coverage.covered:,} | {coverage.total:,} | {coverage.percent:.2f}% |"
        )

    lines.extend(
        [
            "",
            f"## Lowest-covered source files (top {top})",
            "",
            "| File | Covered lines | Total lines | Coverage |",
            "| --- | ---: | ---: | ---: |",
        ]
    )
    for filename, coverage in sorted(files, key=lambda item: (item[1].percent, item[0]))[:top]:
        lines.append(
            f"| `{filename}` | {coverage.covered:,} | {coverage.total:,} | {coverage.percent:.2f}% |"
        )
    lines.append("")
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True, help="cargo llvm-cov JSON file")
    parser.add_argument("--output", type=Path, help="write Markdown here instead of stdout")
    parser.add_argument("--top", type=int, default=10, help="number of low-coverage files to show")
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    if arguments.top < 1:
        raise ValueError("--top must be at least 1")
    try:
        document = json.loads(arguments.input.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"could not read coverage JSON: {error}") from error
    total, crates, files = collect_coverage(document)
    if total.total == 0:
        raise ValueError("coverage JSON contains no executable workspace lines")
    summary = render_summary(total, crates, files, arguments.top)
    if arguments.output is None:
        print(summary, end="")
    else:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(summary, encoding="utf-8")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(f"coverage report failed: {error}") from error
