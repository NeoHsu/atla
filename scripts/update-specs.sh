#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

curl -fL "https://dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json" \
  -o "${repo_root}/specs/jira-v3.json"

curl -fL "https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json" \
  -o "${repo_root}/specs/confluence-v2.json"

curl -fL "https://dac-static.atlassian.com/cloud/confluence/swagger.v3.json" \
  -o "${repo_root}/specs/confluence-v1.json"

"${repo_root}/scripts/jira-v3-partial-spec.js" \
  "${repo_root}/specs/jira-v3.json" \
  "${repo_root}/specs/jira-v3-partial.json"

"${repo_root}/scripts/confluence-v1-partial-spec.js" \
  "${repo_root}/specs/confluence-v1.json" \
  "${repo_root}/specs/confluence-v1-partial.json"
