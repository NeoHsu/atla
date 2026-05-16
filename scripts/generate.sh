#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
generator="${repo_root}/scripts/openapi-generator.sh"
out_root="${ATLA_GENERATED_OUT:-${repo_root}/target/openapi}"
product="all"
in_place=0

usage() {
  cat <<'USAGE'
Usage: scripts/generate.sh [--product jira|confluence|all] [--out DIR] [--in-place]

Generates Rust clients with openapi-generator-cli 7.22.0.

Default output is target/openapi/ so generated code can be inspected before
replacing workspace crates. Use --in-place only when intentionally refreshing
crates/atla-jira-api and crates/atla-confluence-api.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --product)
      product="${2:?missing product}"
      shift 2
      ;;
    --out)
      out_root="${2:?missing output directory}"
      shift 2
      ;;
    --in-place)
      in_place=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "${product}" in
  jira|confluence|all) ;;
  *)
    echo "invalid product: ${product}" >&2
    exit 2
    ;;
esac

if [[ "${in_place}" -eq 1 ]]; then
  out_root="${repo_root}/crates"
fi

fix_generated_manifest() {
  local manifest="$1"

  # openapi-generator 7.22.0 emits a default feature named rustls-tls while the
  # generated feature is named rustls.
  sed -i 's/default = \["rustls-tls"\]/default = ["rustls"]/g' "${manifest}"
}

generate_client() {
  local name="$1"
  local spec="$2"
  local package="$3"
  local out_dir

  if [[ "${in_place}" -eq 1 ]]; then
    out_dir="${out_root}/${package}"
  else
    out_dir="${out_root}/${name}"
  fi

  case "${out_dir}" in
    "${repo_root}/target/"*|"/tmp/"*|"${repo_root}/crates/atla-jira-api"|"${repo_root}/crates/atla-confluence-api") ;;
    *)
      echo "refusing to remove unsafe output directory: ${out_dir}" >&2
      exit 1
      ;;
  esac

  rm -rf "${out_dir}"

  "${generator}" generate \
    -g rust \
    -i "${spec}" \
    -o "${out_dir}" \
    --additional-properties="packageName=${package},packageVersion=0.1.0,library=reqwest,reqwestDefaultFeatures=rustls-tls" \
    --global-property apis,models,supportingFiles

  fix_generated_manifest "${out_dir}/Cargo.toml"
  echo "generated ${name}: ${out_dir}"
}

if [[ "${product}" == "jira" || "${product}" == "all" ]]; then
  generate_client "jira" "${repo_root}/specs/jira-v3.json" "atla-jira-api"
fi

if [[ "${product}" == "confluence" || "${product}" == "all" ]]; then
  generate_client "confluence" "${repo_root}/specs/confluence-v2.json" "atla-confluence-api"
fi
