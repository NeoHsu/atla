#!/usr/bin/env python3
"""Summarize review-relevant changes in checked-in partial OpenAPI specs."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

HTTP_METHODS = {"delete", "get", "head", "options", "patch", "post", "put", "trace"}
SPEC_FILES = (
    ("Jira v3", Path("specs/jira-v3-partial.json")),
    ("Confluence v2", Path("specs/confluence-v2-partial.json")),
    ("Confluence v1", Path("specs/confluence-v1-partial.json")),
)
MAX_DETAIL_ITEMS = 40


@dataclass(frozen=True)
class Snapshot:
    byte_count: int
    sha256: str
    operations: frozenset[str]
    schemas: frozenset[str]
    contracts: frozenset[str]


@dataclass(frozen=True)
class Delta:
    label: str
    path: Path
    old: Snapshot
    new: Snapshot
    lines_added: int
    lines_removed: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="HEAD", help="Git revision to compare against")
    parser.add_argument(
        "--output",
        type=Path,
        help="Write Markdown to this path instead of standard output",
    )
    parser.add_argument(
        "--repo",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help=argparse.SUPPRESS,
    )
    return parser.parse_args()


def git(repo: Path, *arguments: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        ["git", "-C", str(repo), *arguments],
        check=check,
        capture_output=True,
    )


def read_revision_file(repo: Path, revision: str, path: Path) -> bytes:
    result = git(repo, "show", f"{revision}:{path.as_posix()}", check=False)
    if result.returncode != 0:
        message = result.stderr.decode(errors="replace").strip()
        raise RuntimeError(f"cannot read {path} at {revision}: {message}")
    return result.stdout


CONTRACT_SCALAR_KEYS = (
    "$ref",
    "type",
    "format",
    "nullable",
    "readOnly",
    "writeOnly",
    "deprecated",
    "default",
    "const",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "minLength",
    "maxLength",
    "minItems",
    "maxItems",
    "uniqueItems",
    "pattern",
)


def compact_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def resolve_local_ref(document: dict[str, Any], value: Any) -> Any:
    seen: set[str] = set()
    while isinstance(value, dict) and set(value) == {"$ref"}:
        reference = value["$ref"]
        if not isinstance(reference, str) or not reference.startswith("#/") or reference in seen:
            break
        seen.add(reference)
        resolved: Any = document
        try:
            for part in reference[2:].split("/"):
                resolved = resolved[part.replace("~1", "/").replace("~0", "~")]
        except (KeyError, TypeError):
            break
        value = resolved
    return value


def schema_contract_facts(schema: Any, prefix: str) -> set[str]:
    if not isinstance(schema, dict):
        return {f"{prefix} = {compact_json(schema)}"}

    facts: set[str] = set()
    for key in CONTRACT_SCALAR_KEYS:
        if key in schema:
            facts.add(f"{prefix} {key}={compact_json(schema[key])}")
    if "enum" in schema:
        facts.add(f"{prefix} enum={compact_json(schema['enum'])}")

    required = schema.get("required")
    if isinstance(required, list):
        for name in required:
            facts.add(f"{prefix} requires {name}")

    properties = schema.get("properties")
    if isinstance(properties, dict):
        for name, property_schema in properties.items():
            property_prefix = f"{prefix} property {name}"
            facts.add(property_prefix)
            facts.update(schema_contract_facts(property_schema, property_prefix))

    for key in ("items", "additionalProperties", "not"):
        child = schema.get(key)
        if isinstance(child, dict):
            facts.update(schema_contract_facts(child, f"{prefix} {key}"))
        elif key == "additionalProperties" and isinstance(child, bool):
            facts.add(f"{prefix} additionalProperties={str(child).lower()}")

    for key in ("allOf", "anyOf", "oneOf", "prefixItems"):
        children = schema.get(key)
        if isinstance(children, list):
            for index, child in enumerate(children):
                facts.update(schema_contract_facts(child, f"{prefix} {key}[{index}]"))
    return facts


def collect_contract_facts(document: dict[str, Any]) -> frozenset[str]:
    facts: set[str] = set()
    paths = document.get("paths", {})
    if isinstance(paths, dict):
        for route, path_item in paths.items():
            if not isinstance(path_item, dict):
                continue
            path_parameters = path_item.get("parameters", [])
            if not isinstance(path_parameters, list):
                path_parameters = []
            for method, operation in path_item.items():
                if method.lower() not in HTTP_METHODS or not isinstance(operation, dict):
                    continue
                operation_prefix = f"{method.upper()} {route}"
                operation_parameters = operation.get("parameters", [])
                if not isinstance(operation_parameters, list):
                    operation_parameters = []
                raw_parameters = [*path_parameters, *operation_parameters]
                for raw_parameter in raw_parameters:
                    parameter = resolve_local_ref(document, raw_parameter)
                    if not isinstance(parameter, dict):
                        continue
                    location = parameter.get("in", "unknown")
                    name = parameter.get("name", "unknown")
                    parameter_prefix = f"{operation_prefix} parameter {location}:{name}"
                    facts.add(parameter_prefix)
                    facts.add(
                        f"{parameter_prefix} required={str(bool(parameter.get('required'))).lower()}"
                    )
                    if "schema" in parameter:
                        facts.update(
                            schema_contract_facts(
                                parameter["schema"], f"{parameter_prefix} schema"
                            )
                        )

                request_body = resolve_local_ref(document, operation.get("requestBody"))
                if isinstance(request_body, dict):
                    body_prefix = f"{operation_prefix} requestBody"
                    facts.add(
                        f"{body_prefix} required={str(bool(request_body.get('required'))).lower()}"
                    )
                    content = request_body.get("content", {})
                    if isinstance(content, dict):
                        for media_type, media in content.items():
                            if isinstance(media, dict) and "schema" in media:
                                facts.update(
                                    schema_contract_facts(
                                        media["schema"], f"{body_prefix} {media_type}"
                                    )
                                )

                responses = operation.get("responses", {})
                if isinstance(responses, dict):
                    for status, raw_response in responses.items():
                        response = resolve_local_ref(document, raw_response)
                        if not isinstance(response, dict):
                            continue
                        content = response.get("content", {})
                        if not isinstance(content, dict):
                            continue
                        for media_type, media in content.items():
                            if isinstance(media, dict) and "schema" in media:
                                facts.update(
                                    schema_contract_facts(
                                        media["schema"],
                                        f"{operation_prefix} response {status} {media_type}",
                                    )
                                )

    components = document.get("components", {})
    schemas = components.get("schemas", {}) if isinstance(components, dict) else {}
    if isinstance(schemas, dict):
        for name, schema in schemas.items():
            prefix = f"schema {name}"
            facts.add(prefix)
            facts.update(schema_contract_facts(schema, prefix))
    return frozenset(facts)


def parse_snapshot(source: bytes, description: str) -> Snapshot:
    try:
        document: Any = json.loads(source)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"invalid JSON in {description}: {error}") from error
    if not isinstance(document, dict):
        raise RuntimeError(f"OpenAPI document in {description} is not an object")

    operations: set[str] = set()
    paths = document.get("paths", {})
    if isinstance(paths, dict):
        for route, path_item in paths.items():
            if not isinstance(path_item, dict):
                continue
            for method, operation in path_item.items():
                if method.lower() not in HTTP_METHODS or not isinstance(operation, dict):
                    continue
                operation_id = operation.get("operationId")
                if isinstance(operation_id, str) and operation_id:
                    operations.add(operation_id)
                else:
                    operations.add(f"{method.upper()} {route}")

    schemas: set[str] = set()
    components = document.get("components", {})
    if isinstance(components, dict):
        raw_schemas = components.get("schemas", {})
        if isinstance(raw_schemas, dict):
            schemas.update(str(name) for name in raw_schemas)

    return Snapshot(
        byte_count=len(source),
        sha256=hashlib.sha256(source).hexdigest(),
        operations=frozenset(operations),
        schemas=frozenset(schemas),
        contracts=collect_contract_facts(document),
    )


def line_delta(repo: Path, revision: str, path: Path) -> tuple[int, int]:
    result = git(repo, "diff", "--numstat", revision, "--", path.as_posix())
    output = result.stdout.decode().strip()
    if not output:
        return 0, 0
    first_line = output.splitlines()[0]
    added, removed, _ = first_line.split("\t", 2)
    if added == "-" or removed == "-":
        return 0, 0
    try:
        return int(added), int(removed)
    except ValueError as error:
        raise RuntimeError(
            f"unexpected git numstat for {path}: {first_line!r}"
        ) from error


def collect_deltas(repo: Path, revision: str) -> list[Delta]:
    deltas = []
    for label, relative_path in SPEC_FILES:
        current_path = repo / relative_path
        try:
            current = current_path.read_bytes()
        except OSError as error:
            raise RuntimeError(f"cannot read {current_path}: {error}") from error
        previous = read_revision_file(repo, revision, relative_path)
        added, removed = line_delta(repo, revision, relative_path)
        deltas.append(
            Delta(
                label=label,
                path=relative_path,
                old=parse_snapshot(previous, f"{relative_path} at {revision}"),
                new=parse_snapshot(current, str(relative_path)),
                lines_added=added,
                lines_removed=removed,
            )
        )
    return deltas


def format_bytes(count: int) -> str:
    if count < 1024:
        return f"{count} B"
    if count < 1024 * 1024:
        return f"{count / 1024:.1f} KiB"
    return f"{count / (1024 * 1024):.1f} MiB"


def format_transition(old: int, new: int) -> str:
    delta = new - old
    sign = "+" if delta >= 0 else ""
    return f"{old} → {new} ({sign}{delta})"


def render_items(items: set[str]) -> str:
    if not items:
        return "- None"
    ordered = sorted(items)
    visible = ordered[:MAX_DETAIL_ITEMS]
    lines = [f"- `{item}`" for item in visible]
    omitted = len(ordered) - len(visible)
    if omitted:
        lines.append(f"- … {omitted} more omitted")
    return "\n".join(lines)


def render_summary(deltas: list[Delta], revision: str) -> str:
    lines = [
        "## Atlassian partial-spec refresh",
        "",
        f"Generated against `{revision}` by `scripts/spec-diff-summary.py`.",
        "",
        "### Partial-spec summary",
        "",
        "| Spec | Lines | Bytes | Operations | Schemas | New SHA-256 |",
        "| --- | ---: | ---: | ---: | ---: | --- |",
    ]
    for delta in deltas:
        lines.append(
            "| "
            f"`{delta.path}` | +{delta.lines_added} / -{delta.lines_removed} | "
            f"{format_bytes(delta.old.byte_count)} → {format_bytes(delta.new.byte_count)} | "
            f"{format_transition(len(delta.old.operations), len(delta.new.operations))} | "
            f"{format_transition(len(delta.old.schemas), len(delta.new.schemas))} | "
            f"`{delta.new.sha256[:12]}` |"
        )

    lines.extend(["", "### Operation ID changes", ""])
    any_operation_changes = False
    for delta in deltas:
        added = set(delta.new.operations - delta.old.operations)
        removed = set(delta.old.operations - delta.new.operations)
        if not added and not removed:
            continue
        any_operation_changes = True
        lines.extend(
            [
                f"#### {delta.label}",
                "",
                "Added:",
                render_items(added),
                "",
                "Removed:",
                render_items(removed),
                "",
            ]
        )
    if not any_operation_changes:
        lines.extend(["No operation IDs changed.", ""])

    lines.extend(["### Contract changes", ""])
    any_contract_changes = False
    for delta in deltas:
        added = set(delta.new.contracts - delta.old.contracts)
        removed = set(delta.old.contracts - delta.new.contracts)
        if not added and not removed:
            continue
        any_contract_changes = True
        lines.extend(
            [
                f"#### {delta.label}",
                "",
                "Added or changed-to contract facts:",
                render_items(added),
                "",
                "Removed or changed-from contract facts:",
                render_items(removed),
                "",
            ]
        )
    if not any_contract_changes:
        lines.extend(["No parameter, request, response, or schema contract facts changed.", ""])

    lines.extend(
        [
            "### Validation completed",
            "",
            "- `cargo fmt --all -- --check`",
            "- `cargo check --workspace`",
            "- `cargo test --workspace`",
            "",
            "### Reviewer checklist",
            "",
            "- [ ] Review `specs/PATCHES.md` invariants.",
            "- [ ] Review operation additions/removals and partial-spec pruning.",
            "- [ ] Confirm `specs/manifest.json` hashes match the checked-in specs.",
            "- [ ] Confirm generated model conversions still cover changed schemas.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    arguments = parse_args()
    repo = arguments.repo.resolve()
    try:
        deltas = collect_deltas(repo, arguments.base)
        summary = render_summary(deltas, arguments.base)
        if arguments.output:
            arguments.output.parent.mkdir(parents=True, exist_ok=True)
            arguments.output.write_text(summary)
        else:
            print(summary, end="")
    except (OSError, RuntimeError, subprocess.SubprocessError, ValueError) as error:
        print(f"spec-diff-summary: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
