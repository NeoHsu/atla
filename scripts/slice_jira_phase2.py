#!/usr/bin/env python3
from __future__ import annotations

import argparse
import copy
import json
import re
from pathlib import Path
from typing import Any

HTTP_METHODS = {"get", "put", "post", "delete", "patch", "head", "options", "trace"}
REF_RE = re.compile(r"^#/components/(?P<section>[^/]+)/(?P<name>.+)$")

PHASE2_OPERATIONS: list[tuple[str, str]] = [
    ("post", "/rest/api/3/issue"),
    ("get", "/rest/api/3/issue/{issueIdOrKey}"),
    ("put", "/rest/api/3/issue/{issueIdOrKey}"),
    ("delete", "/rest/api/3/issue/{issueIdOrKey}"),
    ("post", "/rest/api/3/issue/{issueIdOrKey}/transitions"),
    ("get", "/rest/api/3/issue/{issueIdOrKey}/transitions"),
    ("put", "/rest/api/3/issue/{issueIdOrKey}/assignee"),
    ("post", "/rest/api/3/issue/{issueIdOrKey}/comment"),
    ("get", "/rest/api/3/issue/{issueIdOrKey}/comment"),
    ("get", "/rest/api/3/project/search"),
    ("get", "/rest/api/3/project/{projectIdOrKey}"),
    # Jira v3 currently uses GET /rest/api/3/search (not POST /issue/search)
    ("get", "/rest/api/3/search"),
]


def walk_refs(node: Any, spec: dict[str, Any], refs_seen: set[str], nodes_seen: set[int]) -> None:
    node_id = id(node)
    if node_id in nodes_seen:
        return
    nodes_seen.add(node_id)

    if isinstance(node, dict):
        ref = node.get("$ref")
        if isinstance(ref, str):
            if ref in refs_seen:
                return
            refs_seen.add(ref)
            match = REF_RE.match(ref)
            if match:
                section = match.group("section")
                name = match.group("name")
                target = spec.get("components", {}).get(section, {}).get(name)
                if target is not None:
                    walk_refs(target, spec, refs_seen, nodes_seen)
            return

        for value in node.values():
            walk_refs(value, spec, refs_seen, nodes_seen)
    elif isinstance(node, list):
        for item in node:
            walk_refs(item, spec, refs_seen, nodes_seen)


def build_sliced_spec(spec: dict[str, Any], operations: list[tuple[str, str]]) -> tuple[dict[str, Any], set[str]]:
    selected_paths: dict[str, Any] = {}
    refs_seen: set[str] = set()
    nodes_seen: set[int] = set()

    for method, path in operations:
        if path not in spec.get("paths", {}):
            raise KeyError(f"Path not found in source spec: {path}")

        path_item = spec["paths"][path]
        if method not in path_item:
            raise KeyError(f"Operation not found in source spec: {method.upper()} {path}")

        selected_path_item = selected_paths.setdefault(path, {})

        for k, v in path_item.items():
            if k not in HTTP_METHODS:
                selected_path_item[k] = copy.deepcopy(v)
                walk_refs(v, spec, refs_seen, nodes_seen)

        selected_path_item[method] = copy.deepcopy(path_item[method])
        walk_refs(path_item[method], spec, refs_seen, nodes_seen)

    components_out: dict[str, dict[str, Any]] = {}
    for ref in sorted(refs_seen):
        match = REF_RE.match(ref)
        if not match:
            continue

        section = match.group("section")
        name = match.group("name")
        source_bucket = spec.get("components", {}).get(section, {})
        if name not in source_bucket:
            continue

        components_out.setdefault(section, {})[name] = copy.deepcopy(source_bucket[name])

    sliced = {
        "openapi": spec["openapi"],
        "info": copy.deepcopy(spec.get("info", {})),
        "servers": copy.deepcopy(spec.get("servers", [])),
        "tags": copy.deepcopy(spec.get("tags", [])),
        "paths": selected_paths,
        "components": components_out,
    }

    if "security" in spec:
        sliced["security"] = copy.deepcopy(spec["security"])
    if "externalDocs" in spec:
        sliced["externalDocs"] = copy.deepcopy(spec["externalDocs"])

    return sliced, refs_seen


def main() -> None:
    parser = argparse.ArgumentParser(description="Slice jira-v3 OpenAPI spec to Phase 2 operations")
    parser.add_argument("--input", default="specs/jira-v3.json")
    parser.add_argument("--output", default="specs/jira-phase2.json")
    args = parser.parse_args()

    source = Path(args.input)
    target = Path(args.output)

    spec = json.loads(source.read_text(encoding="utf-8"))
    sliced, refs_seen = build_sliced_spec(spec, PHASE2_OPERATIONS)

    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(sliced, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    schemas_count = len(sliced.get("components", {}).get("schemas", {}))
    print(f"Wrote {target}")
    print(f"Selected operations: {len(PHASE2_OPERATIONS)}")
    print(f"Referenced components: {len(refs_seen)}")
    print(f"Component schemas kept: {schemas_count}")


if __name__ == "__main__":
    main()
