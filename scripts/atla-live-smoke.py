#!/usr/bin/env python3
"""Deterministic coverage and resource ledger for live atla sandbox smoke tests.

This tool never sends network requests. It makes broad Jira and Confluence live testing auditable
by validating preflight evidence, grouping remote operations, protecting designated targets,
bounding mutations/resources, and refusing to finish while coverage or cleanup is unresolved.
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

SCHEMA_VERSION = 2
RESOURCE_TYPES = (
    "page",
    "blog",
    "attachment",
    "comment",
    "label",
    "space",
    "jira-issue",
    "jira-attachment",
    "jira-comment",
    "jira-link",
    "jira-worklog",
    "jira-sprint",
)
RESOURCE_STATES = ("active", "trashed", "deleted", "purged", "residue")

# Keep every remote operation aligned with crates/atla-cli/src/operation.rs. Tests compare this
# registry to the Rust catalog so a new command cannot silently escape live-smoke classification.
GROUP_OPERATIONS: dict[str, dict[str, str]] = {
    "auth-discovery": {
        "auth.discover": "read",
    },
    "jira-issue-lifecycle": {
        "jira.board.list": "read",
        "jira.board.view": "read",
        "jira.issue.assign": "write",
        "jira.issue.comment.add": "write",
        "jira.issue.comment.delete": "destructive",
        "jira.issue.comment.list": "read",
        "jira.issue.comment.update": "write",
        "jira.issue.create": "write",
        "jira.issue.delete": "destructive",
        "jira.issue.fields": "read",
        "jira.issue.link.add": "write",
        "jira.issue.link.github-commits": "read",
        "jira.issue.link.github-links": "read",
        "jira.issue.link.list": "read",
        "jira.issue.link.remove": "destructive",
        "jira.issue.transition": "write",
        "jira.issue.update": "write",
        "jira.issue.view": "read",
        "jira.issue.worklog.add": "write",
        "jira.issue.worklog.list": "read",
        "jira.project.issue-types": "read",
        "jira.project.list": "read",
        "jira.project.view": "read",
        "jira.sprint.active": "read",
        "jira.sprint.add": "write",
        "jira.sprint.close": "write",
        "jira.sprint.create": "write",
        "jira.sprint.issues": "read",
        "jira.sprint.list": "read",
        "jira.sprint.remove": "write",
        "jira.sprint.start": "write",
        "jira.sprint.view": "read",
    },
    "jira-attachment-lifecycle": {
        "jira.issue.attachment.delete": "destructive",
        "jira.issue.attachment.download": "read",
        "jira.issue.attachment.list": "read",
        "jira.issue.attachment.upload": "write",
    },
    "confluence-page-lifecycle": {
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
    },
    "confluence-attachment-lifecycle": {
        "confluence.attachment.list": "read",
        "confluence.attachment.view": "read",
        "confluence.attachment.upload": "write",
        "confluence.attachment.download": "read",
        "confluence.attachment.delete": "destructive",
    },
    "cql-jql-search": {
        "jira.issue.list": "read",
        "jira.search": "read",
        "confluence.search": "read",
    },
}
OPERATIONS = {
    operation: risk
    for operations in GROUP_OPERATIONS.values()
    for operation, risk in operations.items()
}
OPERATION_GROUPS = {
    operation: group
    for group, operations in GROUP_OPERATIONS.items()
    for operation in operations
}
SPACE_MUTATIONS = {
    "confluence.space.create",
    "confluence.space.update",
    "confluence.space.delete",
}
NON_REVERSIBLE_MUTATIONS = {
    "jira.issue.worklog.add",
    "jira.sprint.add",
    "jira.sprint.close",
    "jira.sprint.create",
    "jira.sprint.remove",
    "jira.sprint.start",
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
    "jira.issue.create": "jira-issue",
    "jira.issue.attachment.upload": "jira-attachment",
    "jira.issue.comment.add": "jira-comment",
    "jira.issue.link.add": "jira-link",
    "jira.issue.worklog.add": "jira-worklog",
    "jira.sprint.create": "jira-sprint",
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
    "jira.issue.delete": "jira-issue",
    "jira.issue.attachment.delete": "jira-attachment",
    "jira.issue.comment.delete": "jira-comment",
    "jira.issue.link.remove": "jira-link",
}
RESOURCE_PARENT_TYPES = {
    "attachment": {"page", "blog"},
    "comment": {"page", "blog"},
    "label": {"page", "blog"},
    "jira-attachment": {"jira-issue"},
    "jira-comment": {"jira-issue"},
    "jira-worklog": {"jira-issue"},
}
MUTATED_RESOURCE_TYPES = {
    "confluence.space.update": "space",
    "confluence.page.update": "page",
    "confluence.page.move": "page",
    "confluence.blog.update": "blog",
    "jira.issue.assign": "jira-issue",
    "jira.issue.comment.update": "jira-comment",
    "jira.issue.transition": "jira-issue",
    "jira.issue.update": "jira-issue",
    "jira.sprint.add": "jira-sprint",
    "jira.sprint.close": "jira-sprint",
    "jira.sprint.remove": "jira-sprint",
    "jira.sprint.start": "jira-sprint",
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
        raise ValueError(f"unknown remote operation `{operation}`")
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

    if args.max_resources < 1 or args.max_mutations < 1:
        raise ValueError("--max-resources and --max-mutations must be positive")
    if args.allow_space_mutations and not args.allow_mutations:
        raise ValueError("--allow-space-mutations requires --allow-mutations")
    if args.allow_space_mutations and not args.temporary_space_key:
        raise ValueError("--allow-space-mutations requires --temporary-space-key")
    if args.allow_residue and not args.allow_mutations:
        raise ValueError("--allow-residue requires --allow-mutations")
    if args.allow_purge and not args.allow_mutations:
        raise ValueError("--allow-purge requires --allow-mutations")

    selected_groups = tuple(dict.fromkeys(args.group or GROUP_OPERATIONS))
    needs_confluence_target = bool(
        {"confluence-page-lifecycle", "confluence-attachment-lifecycle"}
        & set(selected_groups)
    )
    needs_jira_target = bool(
        {"jira-issue-lifecycle", "jira-attachment-lifecycle"}
        & set(selected_groups)
    )
    if needs_confluence_target and not all(
        (args.confluence_baseline, args.target_page, args.space_key)
    ):
        raise ValueError(
            "Confluence lifecycle groups require --confluence-baseline, "
            "--target-page, and --space-key"
        )
    if needs_jira_target and not all(
        (args.jira_baseline, args.target_issue, args.project_key)
    ):
        raise ValueError(
            "Jira lifecycle groups require --jira-baseline, --target-issue, "
            "and --project-key"
        )

    auth_path = Path(args.auth_status)
    auth = read_json(auth_path)
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
    if args.allow_mutations and auth.get("policy_mode") != "read-write":
        raise ValueError(
            "mutation ledger requires auth evidence with policy_mode `read-write`"
        )

    preflight: dict[str, Any] = {
        "authStatus": {
            "path": str(auth_path.resolve()),
            "sha256": digest_file(auth_path),
            "policyMode": auth.get("policy_mode"),
            "apiTarget": auth.get("api_target"),
        }
    }
    if needs_confluence_target:
        baseline_path = Path(args.confluence_baseline)
        baseline = read_json(baseline_path)
        if str(baseline.get("id")) != args.target_page:
            raise ValueError(
                f"baseline page id `{baseline.get('id')}` does not match protected "
                f"target `{args.target_page}`"
            )
        baseline_version = baseline.get("version")
        if isinstance(baseline_version, dict):
            baseline_version = baseline_version.get("number")
        preflight["confluenceBaseline"] = {
            "path": str(baseline_path.resolve()),
            "sha256": digest_file(baseline_path),
            "version": baseline_version,
        }
    if needs_jira_target:
        baseline_path = Path(args.jira_baseline)
        baseline = read_json(baseline_path)
        if str(baseline.get("key")) != args.target_issue:
            raise ValueError(
                f"baseline issue key `{baseline.get('key')}` does not match protected "
                f"target `{args.target_issue}`"
            )
        preflight["jiraBaseline"] = {
            "path": str(baseline_path.resolve()),
            "sha256": digest_file(baseline_path),
            "id": baseline.get("id"),
        }

    coverage = {
        operation: {
            "group": OPERATION_GROUPS[operation],
            "risk": risk,
            "status": "pending",
            "reason": None,
            "evidence": None,
            "failureClass": None,
        }
        for operation, risk in OPERATIONS.items()
    }
    exclusions = dict(parse_exclusion(raw) for raw in args.exclude)
    for operation, group in OPERATION_GROUPS.items():
        if group not in selected_groups:
            exclusions.setdefault(operation, f"smoke group `{group}` not selected")
    if not args.allow_space_mutations:
        for operation in SPACE_MUTATIONS:
            exclusions.setdefault(
                operation, "space mutations not authorized for this run"
            )
    if not args.allow_residue:
        for operation in NON_REVERSIBLE_MUTATIONS:
            exclusions.setdefault(
                operation, "non-reversible sandbox mutation requires --allow-residue"
            )
    for operation, reason in exclusions.items():
        coverage[operation].update(status="skip", reason=reason)

    state = {
        "schemaVersion": SCHEMA_VERSION,
        "runId": args.run_id or datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"),
        "createdAt": utc_now(),
        "updatedAt": utc_now(),
        "status": "running",
        "site": expected_site,
        "profile": args.profile,
        "selectedGroups": list(selected_groups),
        "projectKey": args.project_key,
        "spaceKey": args.space_key,
        "spaceId": args.space_id,
        "temporarySpaceKey": args.temporary_space_key,
        "protectedTargetPageId": args.target_page,
        "protectedTargetIssueKey": args.target_issue,
        "allowMutations": args.allow_mutations,
        "allowSpaceMutations": args.allow_space_mutations,
        "allowPurge": args.allow_purge,
        "allowResidue": args.allow_residue,
        "maxMutationRecords": args.max_mutations,
        "mutationRecords": 0,
        "maxResources": args.max_resources,
        "preflight": preflight,
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
                "selectedGroups": state["selectedGroups"],
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
    if len(state["resources"]) >= state["maxResources"]:
        raise ValueError(
            f"resource budget exhausted ({state['maxResources']}); clean up before adding more"
        )
    if resource_type == "page" and resource_id == state["protectedTargetPageId"]:
        raise ValueError(
            "refusing to track the protected target page as a temporary resource"
        )
    if (
        resource_type == "jira-issue"
        and resource_id == state["protectedTargetIssueKey"]
    ):
        raise ValueError(
            "refusing to track the protected target issue as a temporary resource"
        )
    allowed_parent_types = RESOURCE_PARENT_TYPES.get(resource_type)
    if allowed_parent_types:
        if parent_type not in allowed_parent_types or not parent_id:
            expected = ", ".join(sorted(allowed_parent_types))
            raise ValueError(
                f"resource `{resource_type}` requires tracked parent type {expected} and --parent-id"
            )
        find_resource(state, parent_type, parent_id)
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
        raise ValueError(f"unknown remote operation `{args.operation}`")
    entry = state["coverage"][args.operation]
    if entry["group"] not in state["selectedGroups"] and args.result != "skip":
        raise ValueError(
            f"operation `{args.operation}` belongs to unselected group `{entry['group']}`"
        )
    mutating_attempt = args.result in {"pass", "fail"} and entry["risk"] != "read"
    mutating_pass = args.result == "pass" and entry["risk"] != "read"
    if mutating_pass and not state["allowMutations"]:
        raise ValueError(
            f"cannot record mutation `{args.operation}` in a read-only ledger"
        )
    if mutating_pass and state["mutationRecords"] >= state["maxMutationRecords"]:
        raise ValueError(
            f"mutation budget exhausted ({state['maxMutationRecords']}); cleanup or finish this run"
        )
    if (
        args.operation in SPACE_MUTATIONS
        and mutating_pass
        and not state["allowSpaceMutations"]
    ):
        raise ValueError(
            f"space mutation `{args.operation}` was not authorized for this ledger"
        )
    if (
        args.operation in NON_REVERSIBLE_MUTATIONS
        and mutating_pass
        and not state["allowResidue"]
    ):
        raise ValueError(
            f"non-reversible mutation `{args.operation}` requires --allow-residue"
        )
    if args.result == "fail" and not args.failure_class:
        raise ValueError("fail records require --failure-class")
    if args.result != "fail" and args.failure_class:
        raise ValueError("--failure-class is valid only with --result fail")
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
    mutated_type = MUTATED_RESOURCE_TYPES.get(args.operation)
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
    elif args.result == "pass" and mutated_type:
        if not resources:
            raise ValueError(
                f"passing `{args.operation}` must reference a tracked `--resource {mutated_type}:ID`"
            )
        for resource_type, resource_id in resources:
            if resource_type != mutated_type:
                raise ValueError(
                    f"`{args.operation}` mutates `{mutated_type}`, not `{resource_type}` resources"
                )
            find_resource(state, resource_type, resource_id)
    elif resources:
        raise ValueError(
            "--resource is valid only for passing create, mutation, or destructive operations"
        )

    entry.update(
        status=args.result,
        reason=args.reason,
        evidence=evidence,
        failureClass=args.failure_class,
        recordedAt=utc_now(),
    )
    if mutating_attempt:
        state["mutationRecords"] += 1
    state["history"].append(
        {
            "at": utc_now(),
            "action": "record",
            "operation": args.operation,
            "result": args.result,
            "failureClass": args.failure_class,
        }
    )
    save_state(path, state)
    print(
        json.dumps(
            {
                "operation": args.operation,
                "status": args.result,
                "failureClass": args.failure_class,
                "mutationRecords": state["mutationRecords"],
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
        "active": {"trashed", "deleted", "residue"},
        "trashed": {"purged"},
        "deleted": set(),
        "purged": set(),
        "residue": set(),
    }
    if args.new_state not in allowed[current]:
        raise ValueError(f"invalid resource transition {current} -> {args.new_state}")
    if args.new_state == "purged" and not state["allowPurge"]:
        raise ValueError("ledger was not initialized with --allow-purge")
    if args.new_state == "residue":
        if not state["allowResidue"]:
            raise ValueError("ledger was not initialized with --allow-residue")
        if not args.reason:
            raise ValueError("residue transitions require --reason")
        item["residueReason"] = args.reason
    elif args.reason:
        raise ValueError("--reason is valid only for residue transitions")
    item["state"] = args.new_state
    item["updatedAt"] = utc_now()
    state["history"].append(
        {
            "at": utc_now(),
            "action": "resource-set",
            "resource": f"{args.resource_type}:{args.resource_id}",
            "from": current,
            "to": args.new_state,
            "reason": args.reason,
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
    base = ["atla", "--profile", profile, "--no-input"]
    confluence = [*base, "confluence"]
    jira_issue = [*base, "jira", "issue"]
    if resource_type == "page":
        command = [*confluence, "page", "delete", resource_id]
    elif resource_type == "blog":
        command = [*confluence, "blog", "delete", resource_id]
    elif resource_type == "attachment":
        command = [*confluence, "attachment", "delete", resource_id]
    elif resource_type == "space":
        if status == "trashed":
            return None
        command = [*confluence, "space", "delete", resource_id]
    elif resource_type in {"comment", "label"}:
        parent_type = item.get("parentType")
        parent_id = item.get("parentId")
        if parent_type not in {"page", "blog"} or not parent_id:
            return None
        command = [
            *confluence,
            parent_type,
            resource_type,
            "delete" if resource_type == "comment" else "remove",
            parent_id,
            resource_id,
        ]
    elif resource_type == "jira-issue":
        command = [*jira_issue, "delete", resource_id]
    elif resource_type == "jira-attachment":
        command = [*jira_issue, "attachment", "delete", resource_id]
    elif resource_type == "jira-comment":
        parent_id = item.get("parentId")
        if item.get("parentType") != "jira-issue" or not parent_id:
            return None
        command = [*jira_issue, "comment", "delete", parent_id, resource_id]
    elif resource_type == "jira-link":
        command = [*jira_issue, "link", "remove", resource_id]
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
    residue = [item for item in state["resources"] if item["state"] == "residue"]
    if not unresolved:
        print("# no unresolved temporary resources")
        if residue:
            print(f"# {len(residue)} documented residue resource(s) remain in the sandbox")
        return 0
    for item in reversed(unresolved):
        command = cleanup_command(state, item)
        identifier = f"{item['type']}:{item['id']} ({item['state']})"
        if command:
            print(command)
        elif item["state"] == "trashed" and not state["allowPurge"]:
            print(f"# {identifier}: purge not authorized in this ledger")
        else:
            print(f"# {identifier}: no automatic cleanup command; resolve explicitly")
    return 0


def summary(state: dict[str, Any]) -> dict[str, Any]:
    active, trashed = unresolved_resources(state)
    residue = [item for item in state["resources"] if item["state"] == "residue"]
    return {
        "runId": state["runId"],
        "status": state["status"],
        "site": state["site"],
        "profile": state["profile"],
        "selectedGroups": state["selectedGroups"],
        "counts": operation_counts(state),
        "mutationRecords": state["mutationRecords"],
        "maxMutationRecords": state["maxMutationRecords"],
        "resources": len(state["resources"]),
        "maxResources": state["maxResources"],
        "activeResources": len(active),
        "trashedResources": len(trashed),
        "residueResources": len(residue),
    }


def command_status(args: argparse.Namespace) -> int:
    state = load_state(Path(args.state))
    result = summary(state)
    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print(f"run: {result['runId']} ({result['status']})")
        print(f"target: {result['profile']} @ {result['site']}")
        print(f"groups: {', '.join(result['selectedGroups'])}")
        print(
            "coverage: "
            + ", ".join(f"{key}={value}" for key, value in result["counts"].items())
        )
        print(
            f"mutations: {result['mutationRecords']}/{result['maxMutationRecords']}"
        )
        print(
            "resources: "
            f"{result['resources']}/{result['maxResources']} total, "
            f"active={result['activeResources']}, "
            f"trashed={result['trashedResources']}, "
            f"residue={result['residueResources']}"
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
            [
                {
                    "operation": operation,
                    "risk": risk,
                    "group": OPERATION_GROUPS[operation],
                }
                for operation, risk in OPERATIONS.items()
            ],
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
        "--baseline",
        "--confluence-baseline",
        dest="confluence_baseline",
        help="JSON from a read-only target page view",
    )
    init.add_argument("--jira-baseline", help="JSON from a read-only target issue view")
    init.add_argument("--site", required=True)
    init.add_argument("--profile", required=True)
    init.add_argument("--target-page")
    init.add_argument("--target-issue")
    init.add_argument("--project-key")
    init.add_argument("--space-key")
    init.add_argument("--space-id")
    init.add_argument("--run-id")
    init.add_argument(
        "--group", action="append", choices=tuple(GROUP_OPERATIONS), default=[]
    )
    init.add_argument("--max-resources", type=int, default=50)
    init.add_argument("--max-mutations", type=int, default=50)
    init.add_argument("--allow-mutations", action="store_true")
    init.add_argument("--allow-space-mutations", action="store_true")
    init.add_argument("--temporary-space-key")
    init.add_argument("--allow-purge", action="store_true")
    init.add_argument("--allow-residue", action="store_true")
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
        "--failure-class",
        choices=("api-drift", "cli-regression", "environment"),
    )
    record.add_argument(
        "--resource",
        action="append",
        default=[],
        metavar="TYPE:ID",
        help="created, mutated, or deleted resource; repeat when needed",
    )
    parent_types = sorted(
        {parent for choices in RESOURCE_PARENT_TYPES.values() for parent in choices}
    )
    record.add_argument("--parent-type", choices=parent_types)
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
    resource_add.add_argument("--parent-type", choices=parent_types)
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
    resource_set.add_argument("--reason")
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
        "list-coverage", help="print the complete remote-operation registry"
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
