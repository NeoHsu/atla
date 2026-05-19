#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
generator="${repo_root}/scripts/openapi-generator.sh"
generator_version="${OPENAPI_GENERATOR_VERSION:-7.22.0}"
manifest_path="${repo_root}/specs/manifest.json"
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

replace_in_file() {
  local file="$1"
  local pattern="$2"
  local replacement="$3"

  python - "$file" "$pattern" "$replacement" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
pattern = sys.argv[2]
replacement = sys.argv[3]
text = path.read_text()
updated = re.sub(pattern, replacement, text, flags=re.MULTILINE)
path.write_text(updated)
PY
}

prepend_line_if_missing() {
  local file="$1"
  local line="$2"

  python - "$file" "$line" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
line = sys.argv[2]
text = path.read_text()
prefix = f"{line}\n"
if not text.startswith(prefix):
    text = prefix + text
path.write_text(text)
PY
}

fix_generated_manifest() {
  local manifest="$1"

  # openapi-generator 7.22.0 emits a default feature named rustls-tls while the
  # generated feature is named rustls.
  replace_in_file "${manifest}" 'default = \["rustls-tls"\]' 'default = ["rustls"]'
  replace_in_file "${manifest}" 'reqwest = \{ version = "\^0\.[0-9]+", default-features = false, features = \[[^]]+\] \}' 'reqwest.workspace = true'
  replace_in_file "${manifest}" 'rustls = \["reqwest/rustls"\]' 'rustls = ["reqwest/rustls-tls"]'
}

fix_generated_lints() {
  local lib_rs="$1"

  # The Confluence spec produces a handful of model modules with double
  # underscores. They compile correctly but violate Rust's non_snake_case lint.
  prepend_line_if_missing "${lib_rs}" '#![allow(clippy::too_many_arguments)]'
  prepend_line_if_missing "${lib_rs}" '#![allow(unused_imports)]'
  prepend_line_if_missing "${lib_rs}" '#![allow(non_snake_case)]'
}

update_specs_manifest_generated_at() {
  python - "${manifest_path}" "${generator_version}" <<'PY'
from datetime import datetime, timezone
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
generator_version = sys.argv[2]
timestamp = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
base = {
    "generated_at": timestamp,
    "generator": {
        "tool": "openapi-generator-cli",
        "version": generator_version,
    },
    "specs": {
        "jira": {
            "source_file": "specs/jira-v3-partial.json",
            "partial_spec_script": "scripts/jira-v3-partial-spec.js",
            "description": "Jira REST API v3 (partial, filtered to used endpoints)",
        },
        "confluence": {
            "source_file": "specs/confluence-v2.json",
            "description": "Confluence REST API v2",
        },
        "confluence-v1": {
            "source_file": "specs/confluence-v1-partial.json",
            "partial_spec_script": "scripts/confluence-v1-partial-spec.js",
            "description": "Confluence REST API v1 (partial, filtered and patched)",
        },
    },
}

if path.exists():
    data = json.loads(path.read_text())
else:
    data = {}

data["generated_at"] = timestamp
data["generator"] = {**data.get("generator", {}), **base["generator"]}

specs = data.setdefault("specs", {})
for name, metadata in base["specs"].items():
    specs[name] = {**specs.get(name, {}), **metadata}

path.write_text(json.dumps(data, indent=2) + "\n")
PY
}

format_generated_crate() {
  local manifest="$1"
  local crate_dir
  local edition
  local rust_files=()
  local file

  crate_dir="$(dirname "${manifest}")"

  if [[ "${crate_dir}" == "${repo_root}/crates/"* ]]; then
    if [[ -x "${HOME}/.local/bin/mise" ]]; then
      "${HOME}/.local/bin/mise" exec -- cargo fmt --manifest-path "${manifest}"
      return
    fi

    if command -v mise >/dev/null 2>&1; then
      "$(command -v mise)" exec -- cargo fmt --manifest-path "${manifest}"
      return
    fi

    cargo fmt --manifest-path "${manifest}"
    return
  fi

  edition="$(python - "${manifest}" <<'PY'
from pathlib import Path
import re
import sys

text = Path(sys.argv[1]).read_text()
match = re.search(r'^edition = "([^"]+)"$', text, re.MULTILINE)
print(match.group(1) if match else "2021")
PY
)"

  while IFS= read -r -d '' file; do
    rust_files+=("${file}")
  done < <(find "${crate_dir}/src" -name '*.rs' -print0)

  if [[ "${#rust_files[@]}" -eq 0 ]]; then
    return
  fi

  if [[ -x "${HOME}/.local/bin/mise" ]]; then
    "${HOME}/.local/bin/mise" exec -- rustfmt --edition "${edition}" "${rust_files[@]}"
    return
  fi

  if command -v mise >/dev/null 2>&1; then
    "$(command -v mise)" exec -- rustfmt --edition "${edition}" "${rust_files[@]}"
    return
  fi

  rustfmt --edition "${edition}" "${rust_files[@]}"
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
  update_specs_manifest_generated_at
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

update_specs_manifest_generated_at
