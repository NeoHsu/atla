#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

command -v openapi-generator-cli >/dev/null 2>&1 || {
  echo "openapi-generator-cli is required" >&2
  exit 1
}

openapi-generator-cli generate \
  -g rust \
  -i "${repo_root}/specs/jira-v3.json" \
  -o "${repo_root}/crates/atla-jira-api" \
  --additional-properties=packageName=atla-jira-api,library=reqwest

openapi-generator-cli generate \
  -g rust \
  -i "${repo_root}/specs/confluence-v2.json" \
  -o "${repo_root}/crates/atla-confluence-api" \
  --additional-properties=packageName=atla-confluence-api,library=reqwest

