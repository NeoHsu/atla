#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ACTIONLINT_BIN="${ACTIONLINT_BIN:-}"
if [[ -z "$ACTIONLINT_BIN" ]]; then
	if command -v actionlint >/dev/null 2>&1; then
		ACTIONLINT_BIN="$(command -v actionlint)"
	else
		echo "actionlint not found. Install pinned tools with: mise install" >&2
		exit 127
	fi
fi

"$ACTIONLINT_BIN" .github/workflows/*.yml

unpinned="$({
	grep -HnE '^[[:space:]]*(- )?uses:' .github/workflows/*.yml || true
} | grep -Ev '@[0-9a-f]{40}([[:space:]]|$)' || true)"
if [[ -n "$unpinned" ]]; then
	echo "GitHub Actions must use immutable 40-character commit SHAs:" >&2
	echo "$unpinned" >&2
	exit 1
fi

checkout_count="$({ grep -hE '^[[:space:]]*(- )?uses: actions/checkout@' .github/workflows/*.yml || true; } | wc -l | tr -d ' ')"
persist_count="$({ grep -hF 'persist-credentials: false' .github/workflows/*.yml || true; } | wc -l | tr -d ' ')"
if [[ "$checkout_count" != "$persist_count" ]]; then
	echo "Every checkout action must disable persisted credentials" >&2
	exit 1
fi

CI_WORKFLOW=.github/workflows/ci.yml
RELEASE_WORKFLOW=.github/workflows/release.yml
for workflow in "$CI_WORKFLOW" "$RELEASE_WORKFLOW"; do
	if ! grep -Fq "timeout-minutes:" "$workflow"; then
		echo "$workflow must contain job timeouts" >&2
		exit 1
	fi
done

for required in \
	"gitleaks dir --redact --no-banner ." \
	"cargo deny check" \
	"concurrency:" \
	"zizmorcore/zizmor-action@" \
	"rustsec/audit-check@" \
	"cargo nextest run --workspace --locked" \
	"cargo test --doc --locked -p atla-core -p atla-jira-api"; do
	if ! grep -Fq "$required" "$CI_WORKFLOW"; then
		echo "$CI_WORKFLOW must contain: $required" >&2
		exit 1
	fi
done

for required in \
	"python3 scripts/verify-release-artifacts.py" \
	"actions/attest-build-provenance@" \
	"sha256sum"; do
	if ! grep -Fq "$required" "$RELEASE_WORKFLOW"; then
		echo "$RELEASE_WORKFLOW must contain: $required" >&2
		exit 1
	fi
done

if grep -Eq 'cargo install cargo-(audit|deny|llvm-cov|machete|nextest)' "$CI_WORKFLOW" "$RELEASE_WORKFLOW"; then
	echo "CI must install pinned prebuilt Cargo quality tools instead of compiling them" >&2
	exit 1
fi

printf 'OK: workflow syntax, immutable Action pins, credential hygiene, security gates, and release checks are valid.\n'
