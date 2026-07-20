#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then
	if [[ -n "${ATLA_BUILD_CACHE_DIR:-}" ]]; then
		cache_root="${ATLA_BUILD_CACHE_DIR}"
	elif [[ -n "${XDG_CACHE_HOME:-}" ]]; then
		cache_root="${XDG_CACHE_HOME}/atla"
	elif [[ -n "${HOME:-}" ]]; then
		cache_root="${HOME}/.cache/atla"
	else
		cache_root="${repo_root}/target"
	fi
	export CARGO_TARGET_DIR="${cache_root}/cargo-target"
fi

mkdir -p "${CARGO_TARGET_DIR}"
cd "${repo_root}"
exec cargo check -p atla "$@"
