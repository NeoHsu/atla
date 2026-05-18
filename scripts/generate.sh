#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
generator="${repo_root}/scripts/openapi-generator.sh"
out_root="${ATLA_GENERATED_OUT:-${repo_root}/target/openapi}"
product="all"
in_place=0

usage() {
  cat <<'USAGE'
Usage: scripts/generate.sh [--product jira|confluence|confluence-v1|all] [--out DIR] [--in-place]

Generates Rust clients with openapi-generator-cli 7.22.0.

Default output is target/openapi/ so generated code can be inspected before
replacing workspace crates. Use --in-place only when intentionally refreshing
workspace generated API crates.
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
  jira|confluence|confluence-v1|all) ;;
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

fix_generated_lints() {
  local lib_rs="$1"

  # The Confluence spec produces a handful of model modules with double
  # underscores. They compile correctly but violate Rust's non_snake_case lint.
  if ! grep -q '#!\[allow(non_snake_case)\]' "${lib_rs}"; then
    sed -i '1i#![allow(non_snake_case)]' "${lib_rs}"
  fi

  if ! grep -q '#!\[allow(clippy::all)\]' "${lib_rs}"; then
    sed -i '1i#![allow(clippy::all)]' "${lib_rs}"
  fi
}

format_generated_crate() {
  local manifest="$1"

  if [[ -x "${HOME}/.local/bin/mise" ]]; then
    "${HOME}/.local/bin/mise" exec -- cargo fmt --manifest-path "${manifest}"
    return
  fi

  if command -v mise >/dev/null 2>&1; then
    "$(command -v mise)" exec -- cargo fmt --manifest-path "${manifest}"
    return
  fi

  cargo fmt --manifest-path "${manifest}"
}

remove_standalone_scaffold() {
  local out_dir="$1"

  rm -f "${out_dir}/.gitignore" "${out_dir}/.travis.yml" "${out_dir}/git_push.sh"
}

fix_generated_docs() {
  local out_dir="$1"

  find "${out_dir}" -name '*.md' -exec perl -0pi -e 's/[ \t]+$//mg; s/\n+\z/\n/' {} +
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
    "${repo_root}/target/"*|"/tmp/"*|"${repo_root}/crates/atla-jira-api"|"${repo_root}/crates/atla-confluence-api"|"${repo_root}/crates/atla-confluence-v1-api") ;;
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
  fix_generated_lints "${out_dir}/src/lib.rs"
  format_generated_crate "${out_dir}/Cargo.toml"
  remove_standalone_scaffold "${out_dir}"
  fix_generated_docs "${out_dir}"
  echo "generated ${name}: ${out_dir}"
}

if [[ "${product}" == "jira" || "${product}" == "all" ]]; then
  generate_client "jira" "${repo_root}/specs/jira-v3-partial.json" "atla-jira-api"
fi

if [[ "${product}" == "confluence" || "${product}" == "all" ]]; then
  generate_client "confluence" "${repo_root}/specs/confluence-v2.json" "atla-confluence-api"
fi

if [[ "${product}" == "confluence-v1" || "${product}" == "all" ]]; then
  generate_client "confluence-v1" "${repo_root}/specs/confluence-v1-partial.json" "atla-confluence-v1-api"
fi
