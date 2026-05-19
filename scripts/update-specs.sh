#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest_path="${repo_root}/specs/manifest.json"
generator_version="${OPENAPI_GENERATOR_VERSION:-7.22.0}"

jira_url="https://dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json"
confluence_v2_url="https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json"
confluence_v1_url="https://dac-static.atlassian.com/cloud/confluence/swagger.v3.json"

curl -fL "${jira_url}" \
  -o "${repo_root}/specs/jira-v3.json"

curl -fL "${confluence_v2_url}" \
  -o "${repo_root}/specs/confluence-v2.json"

curl -fL "${confluence_v1_url}" \
  -o "${repo_root}/specs/confluence-v1.json"

"${repo_root}/scripts/jira-v3-partial-spec.js" \
  "${repo_root}/specs/jira-v3.json" \
  "${repo_root}/specs/jira-v3-partial.json"

"${repo_root}/scripts/confluence-v1-partial-spec.js" \
  "${repo_root}/specs/confluence-v1.json" \
  "${repo_root}/specs/confluence-v1-partial.json"

python - "${repo_root}" "${manifest_path}" "${generator_version}" "${jira_url}" "${confluence_v2_url}" "${confluence_v1_url}" <<'PY'
from pathlib import Path
import hashlib
import json
import sys

repo_root = Path(sys.argv[1])
manifest_path = Path(sys.argv[2])
generator_version = sys.argv[3]
jira_url = sys.argv[4]
confluence_v2_url = sys.argv[5]
confluence_v1_url = sys.argv[6]

def sha256_file(relative_path: str) -> str:
    return hashlib.sha256((repo_root / relative_path).read_bytes()).hexdigest()

base = {
    "generator": {
        "tool": "openapi-generator-cli",
        "version": generator_version,
    },
    "specs": {
        "jira": {
            "source_file": "specs/jira-v3-partial.json",
            "source_sha256": sha256_file("specs/jira-v3-partial.json"),
            "upstream_source_file": "specs/jira-v3.json",
            "upstream_url": jira_url,
            "upstream_sha256": sha256_file("specs/jira-v3.json"),
            "partial_spec_script": "scripts/jira-v3-partial-spec.js",
            "description": "Jira REST API v3 (partial, filtered to used endpoints)",
        },
        "confluence": {
            "source_file": "specs/confluence-v2.json",
            "source_sha256": sha256_file("specs/confluence-v2.json"),
            "upstream_source_file": "specs/confluence-v2.json",
            "upstream_url": confluence_v2_url,
            "upstream_sha256": sha256_file("specs/confluence-v2.json"),
            "description": "Confluence REST API v2",
        },
        "confluence-v1": {
            "source_file": "specs/confluence-v1-partial.json",
            "source_sha256": sha256_file("specs/confluence-v1-partial.json"),
            "upstream_source_file": "specs/confluence-v1.json",
            "upstream_url": confluence_v1_url,
            "upstream_sha256": sha256_file("specs/confluence-v1.json"),
            "partial_spec_script": "scripts/confluence-v1-partial-spec.js",
            "description": "Confluence REST API v1 (partial, filtered and patched)",
        },
    },
}

if manifest_path.exists():
    data = json.loads(manifest_path.read_text())
else:
    data = {}

data["generator"] = {**data.get("generator", {}), **base["generator"]}
if "generated_at" not in data:
    data["generated_at"] = None

specs = data.setdefault("specs", {})
for name, metadata in base["specs"].items():
    specs[name] = {**specs.get(name, {}), **metadata}

manifest_path.write_text(json.dumps(data, indent=2) + "\n")
PY
