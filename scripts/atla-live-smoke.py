#!/usr/bin/env python3
"""Deterministic coverage and resource ledger for live atla Confluence smoke tests.

This tool never sends network requests. It makes broad live testing auditable by validating
preflight evidence, tracking every implemented Confluence command leaf, protecting the designated
target page, and refusing to finish while operations or temporary resources remain unresolved.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
RESOURCE_TYPES = ("page", "blog", "attachment", "comment", "label", "space")
RESOURCE_STATES = ("active", "trashed", "deleted", "purged")

# Keep this list aligned with crates/atla-cli/src/operation.rs. A missing entry is visible at
# `finish`, so a smoke run cannot silently call itself complete after testing only a subset.
OPERATIONS: dict[str, str] = {
    "confluence.space.list": "read",
    "confluence.space.view": "read",
    "confluence.space.create": "write",
    "confluence.space.update": "write",
    "confluence.space.delete": "destructive",
    "confluence.page.create": "write",
    "confluence.page.list": "read",
    "confluence.page.view": "read",
    "confluence.page.children": "read",
    "confluence.page.copy": "write",
    "confluence.page.update": "write",
    "confluence.page.delete": "destructive",
    "confluence.page.move": "write",
    "confluence.page.label.list": "read",
    "confluence.page.label.add": "write",
    "confluence.page.label.remove": "destructive",
    "confluence.page.comment.list": "read",
    "confluence.page.comment.add": "write",
    "confluence.page.comment.delete": "destructive",
    "confluence.blog.create": "write",
    "confluence.blog.list": "read",
    "confluence.blog.view": "read",
    "confluence.blog.update": "write",
    "confluence.blog.delete": "destructive",
    "confluence.blog.label.list": "read",
    "confluence.blog.label.add": "write",
    "confluence.blog.label.remove": "destructive",
    "confluence.blog.comment.list": "read",
    "confluence.blog.comment.add": "write",
    "confluence.blog.comment.delete": "destructive",
    "confluence.search": "read",
    "confluence.attachment.list": "read",
    "confluence.attachment.view": "read",
    "confluence.attachment.upload": "write",
    "confluence.attachment.download": "read",
    "confluence.attachment.delete": "destructive",
}
SPACE_MUTATIONS = {
    "confluence.space.create",
    "confluence.space.update",
    "confluence.space.delete",
}
CREATED_RESOURCE_TYPES = {
    "confluence.space.create": "space",
    "confluence.page.create": "page",
    "confluence.page.copy": "page",
    "confluence.page.label.add": "label",
    "confluence.page.comment.add": "comment",
    "confluence.blog.create": "blog",
    "confluence.blog.label.add": "label",
    "confluence.blog.comment.add": "comment",
    "confluence.attachment.upload": "attachment",
}
DESTRUCTIVE_RESOURCE_TYPES = {
    "confluence.space.delete": "space",
    "confluence.page.delete": "page",
    "confluence.page.label.remove": "label",
    "confluence.page.comment.delete": "comment",
    "confluence.blog.delete": "blog",
    "confluence.blog.label.remove": "label",
    "confluence.blog.comment.delete": "comment",
    "confluence.attachment.delete": "attachment",
}


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def normalize_site(value: str) -> str:
    return value.rstrip("/")


def read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"failed to read JSON evidence `{path}`: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"JSON evidence `{path}` must contain an object")
    return value


def digest_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(64 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


def atomic_write(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    if os.name != "nt":
        os.chmod(path.parent, 0o700)
    payload = json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary_path = Path(temporary)
    try:
        if os.name != "nt":
            os.fchmod(descriptor, 0o600)
        with os.fdopen(descriptor, "w") as handle:
            handle.write(payload)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary_path, path)
        if os.name != "nt":
            os.chmod(path, 0o600)
    finally:
        temporary_path.unlink(missing_ok=True)


def load_state(path: Path) -> dict[str, Any]:
    state = read_json(path)
    if state.get("schemaVersion") != SCHEMA_VERSION:
        raise ValueError(
            f"unsupported ledger schemaVersion {state.get('schemaVersion')!r}; expected {SCHEMA_VERSION}"
        )
    return state


def save_state(path: Path, state: dict[str, Any]) -> None:
    state["updatedAt"] = utc_now()
    atomic_write(path, state)


def parse_exclusion(raw: str) -> tuple[str, str]:
    operation, separator, reason = raw.partition("=")
    if not separator or not reason.strip():
        raise ValueError("--exclude must use OPERATION=REASON")
    if operation not in OPERATIONS:
        raise ValueError(f"unknown Confluence operation `{operation}`")
    return operation, reason.strip()


def evidence_record(raw: str) -> dict[str, Any]:
    path = Path(raw)
    if path.is_file():
        return {
            "path": str(path.resolve()),
            "sha256": digest_file(path),
            "bytes": path.stat().st_size,
        }
    if not raw.strip():
        raise ValueError("pass/fail records require non-empty --evidence")
    return {"note": raw.strip()}


def operation_counts(state: dict[str, Any]) -> dict[str, int]:
    counts = {"pending": 0, "pass": 0, "fail": 0, "skip": 0}
    for entry in state["coverage"].values():
        counts[entry["status"]] += 1
    return counts


def unresolved_resources(
    state: dict[str, Any],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    active = [item for item in state["resources"] if item["state"] == "active"]
    trashed = [item for item in state["resources"] if item["state"] == "trashed"]
    return active, trashed


def command_init(args: argparse.Namespace) -> int:
    state_path = Path(args.state)
    if state_path.exists():
        raise ValueError(
            f"ledger already exists: {state_path}; resume it instead of starting over"
        )

    auth_path = Path(args.auth_status)
    baseline_path = Path(args.baseline)
    auth = read_json(auth_path)
    baseline = read_json(baseline_path)
    expected_site = normalize_site(args.site)
    actual_site = normalize_site(str(auth.get("instance", "")))
    if actual_site != expected_site:
        raise ValueError(
            f"auth evidence targets `{actual_site}`, not requested site `{expected_site}`"
        )
    if auth.get("profile") != args.profile:
        raise ValueError(
            f"auth evidence uses profile `{auth.get('profile')}`, not requested profile `{args.profile}`"
        )
    if str(baseline.get("id")) != args.target_page:
        raise ValueError(
            f"baseline page id `{baseline.get('id')}` does not match protected target `{args.target_page}`"
        )
    if args.allow_mutations and auth.get("policy_mode") != "read-write":
        raise ValueError(
            "mutation ledger requires auth evidence with policy_mode `read-write`"
        )
    if args.allow_space_mutations and not args.allow_mutations:
        raise ValueError("--allow-space-mutations requires --allow-mutations")
    if args.allow_space_mutations and not args.temporary_space_key:
        raise ValueError("--allow-space-mutations requires --temporary-space-key")

    coverage = {
        operation: {"risk": risk, "status": "pending", "reason": None, "evidence": None}
        for operation, risk in OPERATIONS.items()
    }
    exclusions = dict(parse_exclusion(raw) for raw in args.exclude)
    if not args.allow_space_mutations:
        for operation in SPACE_MUTATIONS:
            exclusions.setdefault(
                operation, "space mutations not authorized for this run"
            )
    for operation, reason in exclusions.items():
        coverage[operation].update(status="skip", reason=reason)

    baseline_version = baseline.get("version")
    if isinstance(baseline_version, dict):
        baseline_version = baseline_version.get("number")
    state = {
        "schemaVersion": SCHEMA_VERSION,
        "runId": args.run_id or datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"),
        "createdAt": utc_now(),
        "updatedAt": utc_now(),
        "status": "running",
        "site": expected_site,
        "profile": args.profile,
        "spaceKey": args.space_key,
        "spaceId": args.space_id,
        "temporarySpaceKey": args.temporary_space_key,
        "protectedTargetPageId": args.target_page,
        "allowMutations": args.allow_mutations,
        "allowSpaceMutations": args.allow_space_mutations,
        "allowPurge": args.allow_purge,
        "preflight": {
            "authStatus": {
                "path": str(auth_path.resolve()),
                "sha256": digest_file(auth_path),
                "policyMode": auth.get("policy_mode"),
                "apiTarget": auth.get("api_target"),
            },
            "baseline": {
                "path": str(baseline_path.resolve()),
                "sha256": digest_file(baseline_path),
                "version": baseline_version,
            },
        },
        "coverage": coverage,
        "resources": [],
        "history": [{"at": utc_now(), "action": "init"}],
    }
    save_state(state_path, state)
    print(
        json.dumps(
            {
                "state": str(state_path),
                "runId": state["runId"],
                "counts": operation_counts(state),
            }
        )
    )
    return 0


def find_resource(
    state: dict[str, Any], resource_type: str, resource_id: str
) -> dict[str, Any]:
    matches = [
        item
        for item in state["resources"]
        if item["type"] == resource_type and item["id"] == resource_id
    ]
    if not matches:
        raise ValueError(
            f"resource `{resource_type}:{resource_id}` is not in this run's ledger"
        )
    return matches[0]


def parse_resource_spec(raw: str) -> tuple[str, str]:
    resource_type, separator, resource_id = raw.partition(":")
    if not separator or resource_type not in RESOURCE_TYPES or not resource_id:
        raise ValueError("--resource must use TYPE:ID with a supported resource type")
    return resource_type, resource_id


def add_resource(
    state: dict[str, Any],
    resource_type: str,
    resource_id: str,
    *,
    parent_type: str | None,
    parent_id: str | None,
    title: str | None,
    source_operation: str | None,
    evidence: dict[str, Any],
) -> dict[str, Any]:
    if not state["allowMutations"]:
        raise ValueError("cannot add temporary resources to a read-only ledger")
    if resource_type == "page" and resource_id == state["protectedTargetPageId"]:
        raise ValueError(
            "refusing to track the protected target page as a temporary resource"
        )
    try:
        find_resource(state, resource_type, resource_id)
    except ValueError:
        pass
    else:
        raise ValueError(f"resource `{resource_type}:{resource_id}` is already tracked")
    item = {
        "type": resource_type,
        "id": resource_id,
        "state": "active",
        "parentType": parent_type,
        "parentId": parent_id,
        "title": title,
        "sourceOperation": source_operation,
        "evidence": evidence,
        "createdAt": utc_now(),
        "updatedAt": utc_now(),
    }
    state["resources"].append(item)
    state["history"].append(
        {
            "at": utc_now(),
            "action": "resource-add",
            "resource": f"{resource_type}:{resource_id}",
        }
    )
    return item


def command_record(args: argparse.Namespace) -> int:
    path = Path(args.state)
    state = load_state(path)
    if args.operation not in OPERATIONS:
        raise ValueError(f"unknown Confluence operation `{args.operation}`")
    entry = state["coverage"][args.operation]
    if (
        args.result == "pass"
        and entry["risk"] != "read"
        and not state["allowMutations"]
    ):
        raise ValueError(
            f"cannot record mutation `{args.operation}` in a read-only ledger"
        )
    if (
        args.operation in SPACE_MUTATIONS
        and args.result == "pass"
        and not state["allowSpaceMutations"]
    ):
        raise ValueError(
            f"space mutation `{args.operation}` was not authorized for this ledger"
        )
    if args.result == "skip":
        if not args.reason:
            raise ValueError("skip records require --reason")
        if args.resource:
            raise ValueError("skip records cannot add or reference resources")
        evidence = None
    else:
        evidence = evidence_record(args.evidence)

    resources = [parse_resource_spec(raw) for raw in args.resource]
    created_type = CREATED_RESOURCE_TYPES.get(args.operation)
    destructive_type = DESTRUCTIVE_RESOURCE_TYPES.get(args.operation)
    added: list[dict[str, Any]] = []
    if args.result == "pass" and created_type:
        if evidence is None:
            raise ValueError("passing resource creation requires evidence")
        if not resources:
            raise ValueError(
                f"passing `{args.operation}` must record at least one `--resource {created_type}:ID`"
            )
        for resource_type, resource_id in resources:
            if resource_type != created_type:
                raise ValueError(
                    f"`{args.operation}` creates `{created_type}`, not `{resource_type}` resources"
                )
            added.append(
                add_resource(
                    state,
                    resource_type,
                    resource_id,
                    parent_type=args.parent_type,
                    parent_id=args.parent_id,
                    title=args.title,
                    source_operation=args.operation,
                    evidence=evidence,
                )
            )
    elif args.result == "pass" and destructive_type:
        if not resources:
            raise ValueError(
                f"passing `{args.operation}` must reference a tracked `--resource {destructive_type}:ID`"
            )
        for resource_type, resource_id in resources:
            if resource_type != destructive_type:
                raise ValueError(
                    f"`{args.operation}` deletes `{destructive_type}`, not `{resource_type}` resources"
                )
            find_resource(state, resource_type, resource_id)
    elif resources:
        raise ValueError(
            "--resource is valid only for passing create/add/upload or destructive operations"
        )

    entry.update(
        status=args.result, reason=args.reason, evidence=evidence, recordedAt=utc_now()
    )
    state["history"].append(
        {
            "at": utc_now(),
            "action": "record",
            "operation": args.operation,
            "result": args.result,
        }
    )
    save_state(path, state)
    print(
        json.dumps(
            {
                "operation": args.operation,
                "status": args.result,
                "resourcesAdded": added,
            }
        )
    )
    return 0


def command_resource_add(args: argparse.Namespace) -> int:
    path = Path(args.state)
    state = load_state(path)
    item = add_resource(
        state,
        args.resource_type,
        args.resource_id,
        parent_type=args.parent_type,
        parent_id=args.parent_id,
        title=args.title,
        source_operation=None,
        evidence=evidence_record(args.evidence),
    )
    save_state(path, state)
    print(json.dumps(item))
    return 0


def command_resource_set(args: argparse.Namespace) -> int:
    path = Path(args.state)
    state = load_state(path)
    item = find_resource(state, args.resource_type, args.resource_id)
    current = item["state"]
    allowed = {
        "active": {"trashed", "deleted"},
        "trashed": {"purged"},
        "deleted": set(),
        "purged": set(),
    }
    if args.new_state not in allowed[current]:
        raise ValueError(f"invalid resource transition {current} -> {args.new_state}")
    if args.new_state == "purged" and not state["allowPurge"]:
        raise ValueError("ledger was not initialized with --allow-purge")
    item["state"] = args.new_state
    item["updatedAt"] = utc_now()
    state["history"].append(
        {
            "at": utc_now(),
            "action": "resource-set",
            "resource": f"{args.resource_type}:{args.resource_id}",
            "from": current,
            "to": args.new_state,
        }
    )
    save_state(path, state)
    print(json.dumps(item))
    return 0


def cleanup_command(state: dict[str, Any], item: dict[str, Any]) -> str | None:
    profile = state["profile"]
    resource_type = item["type"]
    resource_id = item["id"]
    status = item["state"]
    base = ["atla", "--profile", profile, "--no-input", "confluence"]
    if resource_type == "page":
        command = [*base, "page", "delete", resource_id]
    elif resource_type == "blog":
        command = [*base, "blog", "delete", resource_id]
    elif resource_type == "attachment":
        command = [*base, "attachment", "delete", resource_id]
    elif resource_type == "space":
        if status == "trashed":
            return None
        command = [*base, "space", "delete", resource_id]
    elif resource_type in {"comment", "label"}:
        parent_type = item.get("parentType")
        parent_id = item.get("parentId")
        if parent_type not in {"page", "blog"} or not parent_id:
            return None
        command = [
            *base,
            parent_type,
            resource_type,
            "delete" if resource_type == "comment" else "remove",
        ]
        command.extend([parent_id, resource_id])
    else:
        return None
    if status == "trashed":
        if (
            resource_type not in {"page", "blog", "attachment"}
            or not state["allowPurge"]
        ):
            return None
        command.append("--purge")
    command.append("--yes")
    return shlex.join(command)


def command_cleanup_commands(args: argparse.Namespace) -> int:
    state = load_state(Path(args.state))
    unresolved = [
        item for item in state["resources"] if item["state"] in {"active", "trashed"}
    ]
    if not unresolved:
        print("# no unresolved temporary resources")
        return 0
    for item in reversed(unresolved):
        command = cleanup_command(state, item)
        identifier = f"{item['type']}:{item['id']} ({item['state']})"
        if command:
            print(command)
        elif item["state"] == "trashed" and not state["allowPurge"]:
            print(f"# {identifier}: purge not authorized in this ledger")
        else:
            print(
                f"# {identifier}: manual cleanup required; parent metadata is incomplete"
            )
    return 0


def summary(state: dict[str, Any]) -> dict[str, Any]:
    active, trashed = unresolved_resources(state)
    return {
        "runId": state["runId"],
        "status": state["status"],
        "site": state["site"],
        "profile": state["profile"],
        "counts": operation_counts(state),
        "activeResources": len(active),
        "trashedResources": len(trashed),
    }


def command_status(args: argparse.Namespace) -> int:
    state = load_state(Path(args.state))
    result = summary(state)
    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print(f"run: {result['runId']} ({result['status']})")
        print(f"target: {result['profile']} @ {result['site']}")
        print(
            "coverage: "
            + ", ".join(f"{key}={value}" for key, value in result["counts"].items())
        )
        print(
            f"resources: active={result['activeResources']}, trashed={result['trashedResources']}"
        )
    return 0


def command_finish(args: argparse.Namespace) -> int:
    path = Path(args.state)
    state = load_state(path)
    counts = operation_counts(state)
    active, trashed = unresolved_resources(state)
    if counts["pending"]:
        print(
            f"finish blocked: {counts['pending']} operation(s) are still pending",
            file=sys.stderr,
        )
        return 2
    if counts["fail"]:
        print(f"finish blocked: {counts['fail']} operation(s) failed", file=sys.stderr)
        return 1
    if active:
        print(
            f"finish blocked: {len(active)} temporary resource(s) are still active",
            file=sys.stderr,
        )
        return 3
    if trashed:
        print(
            f"finish blocked: {len(trashed)} temporary resource(s) still require purge",
            file=sys.stderr,
        )
        return 4
    state["status"] = "complete"
    state["completedAt"] = utc_now()
    state["history"].append({"at": utc_now(), "action": "finish"})
    save_state(path, state)
    print(json.dumps(summary(state), ensure_ascii=False))
    return 0


def command_list_coverage(args: argparse.Namespace) -> int:
    del args
    print(
        json.dumps(
            [{"operation": key, "risk": value} for key, value in OPERATIONS.items()],
            indent=2,
        )
    )
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    init = subparsers.add_parser("init", help="create a protected smoke-test ledger")
    init.add_argument("--state", required=True)
    init.add_argument(
        "--auth-status",
        required=True,
        help="JSON from `atla --output json auth status`",
    )
    init.add_argument(
        "--baseline", required=True, help="JSON from a read-only target page view"
    )
    init.add_argument("--site", required=True)
    init.add_argument("--profile", required=True)
    init.add_argument("--target-page", required=True)
    init.add_argument("--space-key", required=True)
    init.add_argument("--space-id")
    init.add_argument("--run-id")
    init.add_argument("--allow-mutations", action="store_true")
    init.add_argument("--allow-space-mutations", action="store_true")
    init.add_argument("--temporary-space-key")
    init.add_argument("--allow-purge", action="store_true")
    init.add_argument(
        "--exclude", action="append", default=[], metavar="OPERATION=REASON"
    )
    init.set_defaults(handler=command_init)

    record = subparsers.add_parser("record", help="record one operation result")
    record.add_argument("--state", required=True)
    record.add_argument("--operation", required=True, choices=sorted(OPERATIONS))
    record.add_argument("--result", required=True, choices=("pass", "fail", "skip"))
    record.add_argument("--evidence", default="")
    record.add_argument("--reason")
    record.add_argument(
        "--resource",
        action="append",
        default=[],
        metavar="TYPE:ID",
        help="created or deleted resource; repeat for multi-resource operations",
    )
    record.add_argument("--parent-type", choices=("page", "blog"))
    record.add_argument("--parent-id")
    record.add_argument("--title")
    record.set_defaults(handler=command_record)

    resource_add = subparsers.add_parser(
        "resource-add", help="track a created temporary resource"
    )
    resource_add.add_argument("--state", required=True)
    resource_add.add_argument(
        "--type", dest="resource_type", required=True, choices=RESOURCE_TYPES
    )
    resource_add.add_argument("--id", dest="resource_id", required=True)
    resource_add.add_argument("--parent-type", choices=("page", "blog"))
    resource_add.add_argument("--parent-id")
    resource_add.add_argument("--title")
    resource_add.add_argument(
        "--evidence",
        required=True,
        help="read-back evidence proving the ambiguous mutation created this resource",
    )
    resource_add.set_defaults(handler=command_resource_add)

    resource_set = subparsers.add_parser(
        "resource-set", help="record cleanup state transition"
    )
    resource_set.add_argument("--state", required=True)
    resource_set.add_argument(
        "--type", dest="resource_type", required=True, choices=RESOURCE_TYPES
    )
    resource_set.add_argument("--id", dest="resource_id", required=True)
    resource_set.add_argument(
        "--to", dest="new_state", required=True, choices=RESOURCE_STATES[1:]
    )
    resource_set.set_defaults(handler=command_resource_set)

    cleanup = subparsers.add_parser(
        "cleanup-commands", help="print commands only for tracked resources"
    )
    cleanup.add_argument("--state", required=True)
    cleanup.set_defaults(handler=command_cleanup_commands)

    status = subparsers.add_parser("status", help="show coverage and cleanup status")
    status.add_argument("--state", required=True)
    status.add_argument("--json", action="store_true")
    status.set_defaults(handler=command_status)

    finish = subparsers.add_parser(
        "finish", help="fail closed on missing coverage or cleanup"
    )
    finish.add_argument("--state", required=True)
    finish.set_defaults(handler=command_finish)

    coverage = subparsers.add_parser(
        "list-coverage", help="print the complete command-leaf registry"
    )
    coverage.set_defaults(handler=command_list_coverage)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return int(args.handler(args))
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
