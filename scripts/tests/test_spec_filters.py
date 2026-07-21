from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FILTER_CASES = (
    (
        "scripts/jira-v3-partial-spec.js",
        "specs/jira-v3.json",
        "specs/jira-v3-partial.json",
    ),
    (
        "scripts/confluence-v1-partial-spec.js",
        "specs/confluence-v1.json",
        "specs/confluence-v1-partial.json",
    ),
    (
        "scripts/confluence-v2-partial-spec.js",
        "specs/confluence-v2.json",
        "specs/confluence-v2-partial.json",
    ),
)


class SpecFilterTests(unittest.TestCase):
    def test_filters_reproduce_checked_in_partial_specs(self) -> None:
        node = shutil.which("node")
        if node is None:
            self.fail("Node.js is required to verify partial-spec filters")
            return

        with tempfile.TemporaryDirectory() as directory:
            output_root = Path(directory)
            for script, source, expected in FILTER_CASES:
                generated = output_root / Path(expected).name
                completed = subprocess.run(
                    [node, str(ROOT / script), str(ROOT / source), str(generated)],
                    check=False,
                    capture_output=True,
                    text=True,
                    cwd=ROOT,
                    timeout=30,
                )
                self.assertEqual(
                    completed.returncode,
                    0,
                    f"{script} failed:\n{completed.stderr}",
                )
                if generated.read_bytes() != (ROOT / expected).read_bytes():
                    self.fail(
                        f"{script} does not reproduce {expected}; rerun the filter and review its patch invariants"
                    )

    def test_manifest_hashes_match_checked_in_sources(self) -> None:
        path = ROOT / "specs/manifest.json"
        try:
            manifest = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            self.fail(f"failed to read {path}: {error}")
            return

        for name, metadata in manifest["specs"].items():
            for file_key, hash_key in (
                ("source_file", "source_sha256"),
                ("upstream_source_file", "upstream_sha256"),
            ):
                source = ROOT / metadata[file_key]
                actual = hashlib.sha256(source.read_bytes()).hexdigest()
                self.assertEqual(
                    actual,
                    metadata[hash_key],
                    f"{name} {hash_key} does not match {metadata[file_key]}",
                )

    def test_jira_project_type_key_remains_open_ended(self) -> None:
        path = ROOT / "specs/jira-v3-partial.json"
        try:
            partial = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            self.fail(f"failed to read {path}: {error}")
            return
        project_type = partial["components"]["schemas"]["Project"]["properties"][
            "projectTypeKey"
        ]
        self.assertEqual(project_type, {"type": "string"})


if __name__ == "__main__":
    unittest.main()
