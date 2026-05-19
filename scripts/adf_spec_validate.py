#!/usr/bin/env python3
import json
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
ATLA = ROOT / "target" / "release" / "atla"


@dataclass
class Case:
    issue: str
    summary: str
    markdown: str


CASES = [
    Case(
        "A",
        "taskList attrs.localId",
        "- [ ] unchecked task\n- [x] checked task",
    ),
    Case(
        "B",
        "taskItem localId reuse",
        "- [ ] first list item\n- [x] second list item\n\nParagraph\n\n- [ ] third list item",
    ),
    Case(
        "C",
        "heading inside blockquote",
        "> # Quoted heading\n>\n> body",
    ),
    Case(
        "D",
        "empty code block",
        "```\n```",
    ),
    Case(
        "E",
        "underscore italic",
        "_italic_",
    ),
    Case(
        "F",
        "combined inline marks",
        "**_bold italic_**",
    ),
    Case(
        "G",
        "double underscore bold",
        "__bold__",
    ),
]


def run_cmd(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, text=True, capture_output=True)


def validate_adf(value: Any) -> list[dict[str, Any]]:
    issues: list[dict[str, Any]] = []

    def push(path: str, message: str) -> None:
        issues.append({"path": path, "message": message})

    def walk(node: Any, path: str, parent_type: str | None = None) -> None:
        if isinstance(node, dict):
            node_type = node.get("type")

            if node_type == "taskList":
                attrs = node.get("attrs")
                if not isinstance(attrs, dict) or not isinstance(attrs.get("localId"), str):
                    push(path, "taskList requires attrs.localId:string")

            if node_type == "taskItem":
                attrs = node.get("attrs")
                if not isinstance(attrs, dict):
                    push(path, "taskItem requires attrs")
                else:
                    if not isinstance(attrs.get("localId"), str):
                        push(path, "taskItem requires attrs.localId:string")
                    if attrs.get("state") not in {"TODO", "DONE"}:
                        push(path, "taskItem requires attrs.state in {TODO,DONE}")

            if node_type == "blockquote":
                for index, child in enumerate(node.get("content", []) or []):
                    child_type = child.get("type") if isinstance(child, dict) else None
                    if child_type not in {
                        "paragraph",
                        "orderedList",
                        "bulletList",
                        "codeBlock",
                        "mediaSingle",
                        "mediaGroup",
                        "extension",
                    }:
                        push(
                            f"{path}.content[{index}]",
                            f"blockquote child `{child_type}` is not allowed by schema",
                        )

            if node_type == "codeBlock":
                content = node.get("content", [])
                if isinstance(content, list):
                    for index, child in enumerate(content):
                        if not isinstance(child, dict) or child.get("type") != "text":
                            push(
                                f"{path}.content[{index}]",
                                "codeBlock content must contain only text nodes",
                            )
                            continue
                        marks = child.get("marks")
                        if isinstance(marks, list) and marks:
                            push(
                                f"{path}.content[{index}]",
                                "codeBlock text nodes must not contain marks",
                            )
                        if not isinstance(child.get("text"), str) or len(child["text"]) < 1:
                            push(
                                f"{path}.content[{index}]",
                                "codeBlock text node requires non-empty text",
                            )

            if node_type == "text":
                text = node.get("text")
                if not isinstance(text, str) or len(text) < 1:
                    push(path, "text node requires non-empty text")

            if node_type == "inlineCard":
                attrs = node.get("attrs")
                if not isinstance(attrs, dict) or not isinstance(attrs.get("url"), str):
                    push(path, "inlineCard requires attrs.url:string")
                if "content" in node:
                    push(path, "inlineCard must not contain content")

            if node_type == "orderedList":
                attrs = node.get("attrs", {})
                order = attrs.get("order") if isinstance(attrs, dict) else None
                if isinstance(order, (int, float)) and order < 0:
                    push(path, "orderedList attrs.order must be >= 0")

            for key, child in node.items():
                child_path = f"{path}.{key}" if path else key
                walk(child, child_path, node_type if isinstance(node_type, str) else parent_type)
        elif isinstance(node, list):
            for index, child in enumerate(node):
                child_path = f"{path}[{index}]"
                walk(child, child_path, parent_type)

    walk(value, "$")
    return issues


def main() -> int:
    if not ATLA.exists():
        print(json.dumps({"error": f"missing CLI binary: {ATLA}"}))
        return 1

    created: list[str] = []
    scratch_files: list[Path] = []
    results: list[dict[str, Any]] = []
    stamp = int(time.time())

    try:
        for case in CASES:
            summary = f"[ADF-SPEC] {case.issue} {case.summary} {stamp}"
            scratch = ROOT / "scripts" / f".adf_spec_{case.issue}_{stamp}.md"
            scratch.write_text(case.markdown, encoding="utf-8")
            scratch_files.append(scratch)
            create = run_cmd(
                [
                    str(ATLA),
                    "jira",
                    "issue",
                    "create",
                    "--project",
                    "DEMO",
                    "--type",
                    "Task",
                    "--summary",
                    summary,
                    "--description-file",
                    str(scratch),
                    "--no-input",
                    "--output",
                    "json",
                ]
            )

            record: dict[str, Any] = {
                "issue": case.issue,
                "summary": summary,
                "markdown": case.markdown,
                "create_exit_code": create.returncode,
                "create_stdout": create.stdout.strip(),
                "create_stderr": create.stderr.strip(),
            }

            if create.returncode == 0:
                payload = json.loads(create.stdout)
                key = payload["key"]
                created.append(key)
                record["key"] = key

                view = run_cmd(
                    [str(ATLA), "jira", "issue", "view", key, "--output", "json"]
                )
                record["view_exit_code"] = view.returncode
                record["view_stderr"] = view.stderr.strip()

                if view.returncode == 0:
                    issue = json.loads(view.stdout)
                    description = issue.get("fields", {}).get("description")
                    record["stored_adf"] = description
                    record["stored_validation_errors"] = validate_adf(description)
                else:
                    record["stored_adf"] = None
                    record["stored_validation_errors"] = []
            results.append(record)
    finally:
        for scratch in scratch_files:
            scratch.unlink(missing_ok=True)
        cleanup: list[dict[str, Any]] = []
        for key in created:
            deleted = run_cmd([str(ATLA), "jira", "issue", "delete", key, "--yes"])
            cleanup.append(
                {
                    "key": key,
                    "delete_exit_code": deleted.returncode,
                    "delete_stderr": deleted.stderr.strip(),
                }
            )
        print(json.dumps({"cases": results, "cleanup": cleanup}, indent=2))

    return 0


if __name__ == "__main__":
    sys.exit(main())
