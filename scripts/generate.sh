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

  python3 - "$file" "$pattern" "$replacement" <<'PY'
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

  python3 - "$file" "$line" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
line = sys.argv[2]
text = path.read_text()
if line not in text:
    text = f"{line}\n" + text
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
  # Suppress all warnings so that `cargo clippy -- -D warnings` on dependents
  # does not treat generated-code lint warnings as errors.
  prepend_line_if_missing "${lib_rs}" '#![allow(clippy::too_many_arguments)]'
  prepend_line_if_missing "${lib_rs}" '#![allow(unused_imports)]'
  prepend_line_if_missing "${lib_rs}" '#![allow(non_snake_case)]'
  prepend_line_if_missing "${lib_rs}" '#![allow(warnings)]'
}

update_specs_manifest_generated_at() {
  python3 - "${manifest_path}" "${generator_version}" <<'PY'
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

  edition="$(python3 - "${manifest}" <<'PY'
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

# Apply manual patches to atla-jira-api after generation.
# These cover endpoints and type fixes not representable in the partial spec.
patch_jira_api_crate() {
  local out_dir="$1"

  # project.rs: use Option<String> for projectTypeKey to handle unknown variants like "product_discovery"
  python3 - "${out_dir}/src/models/project.rs" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
text = text.replace(
    '    pub project_type_key: Option<ProjectTypeKey>,',
    '    /// The project type key as a raw string to handle unknown variants (e.g. "product_discovery").\n'
    '    pub project_type_key: Option<String>,'
)
path.write_text(text)
PY

  # mod.rs: declare manually-maintained modules; only add a declaration when the file is present
  python3 - "${out_dir}" <<'PY'
from pathlib import Path
import sys
out = Path(sys.argv[1])
mod_rs = out / 'src/apis/mod.rs'
text = mod_rs.read_text()
if (out / 'src/apis/agile_boards_api.rs').exists() and 'pub mod agile_boards_api;' not in text:
    text = text.replace(
        'pub mod issue_attachments_api;',
        'pub mod agile_boards_api;\npub mod agile_sprints_api;\npub mod issue_attachments_api;',
    )
if (out / 'src/apis/issue_worklogs_api.rs').exists() and 'pub mod issue_worklogs_api;' not in text:
    text = text.replace(
        'pub mod issues_api;',
        'pub mod issue_worklogs_api;\npub mod issues_api;',
    )
if (out / 'src/apis/users_api.rs').exists() and 'pub mod users_api;' not in text:
    text = text.replace(
        'pub mod projects_api;\n\npub mod configuration;',
        'pub mod projects_api;\npub mod users_api;\n\npub mod configuration;',
    )
mod_rs.write_text(text)
PY

  # issue_attachments_api.rs: insert RemoveAttachmentError struct and append remove_attachment function
  python3 - "${out_dir}/src/apis/issue_attachments_api.rs" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
anchor = 'pub enum GetAttachmentError {\n    UnknownValue(serde_json::Value),\n}\n'
if anchor in text and 'RemoveAttachmentError' not in text:
    insert = (
        '\n/// struct for typed errors of method [`remove_attachment`]\n'
        '#[derive(Debug, Clone, Serialize, Deserialize)]\n'
        '#[serde(untagged)]\n'
        'pub enum RemoveAttachmentError {\n'
        '    UnknownValue(serde_json::Value),\n'
        '}\n'
    )
    text = text.replace(anchor, anchor + insert, 1)
if 'pub async fn remove_attachment' not in text:
    text = text.rstrip('\n') + '\n' + '''\

pub async fn remove_attachment(
    configuration: &configuration::Configuration,
    id: &str,
) -> Result<(), Error<RemoveAttachmentError>> {
    let p_path_id = id;

    let uri_str = format!(
        "{}/rest/api/3/attachment/{id}",
        configuration.base_path,
        id = crate::apis::urlencode(p_path_id)
    );
    let mut req_builder = configuration
        .client
        .request(reqwest::Method::DELETE, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.oauth_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    if let Some(ref auth_conf) = configuration.basic_auth {
        req_builder = req_builder.basic_auth(auth_conf.0.to_owned(), auth_conf.1.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<RemoveAttachmentError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}
'''
path.write_text(text)
PY

  # issues_api.rs: append SetAssigneeError struct and set_assignee function
  python3 - "${out_dir}/src/apis/issues_api.rs" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
if 'SetAssigneeError' not in text:
    text = text.rstrip('\n') + '\n' + '''\

/// struct for typed errors of method [`set_assignee`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SetAssigneeError {
    UnknownValue(serde_json::Value),
}

pub async fn set_assignee(
    configuration: &configuration::Configuration,
    issue_id_or_key: &str,
    account_id: Option<String>,
) -> Result<(), Error<SetAssigneeError>> {
    let p_path_issue_id_or_key = issue_id_or_key;

    let uri_str = format!(
        "{}/rest/api/3/issue/{issueIdOrKey}/assignee",
        configuration.base_path,
        issueIdOrKey = crate::apis::urlencode(p_path_issue_id_or_key)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::PUT, &uri_str);

    req_builder = req_builder.json(&serde_json::json!({ "accountId": account_id }));

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.oauth_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    if let Some(ref auth_conf) = configuration.basic_auth {
        req_builder = req_builder.basic_auth(auth_conf.0.to_owned(), auth_conf.1.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<SetAssigneeError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}
'''
path.write_text(text)
PY
}

generate_client() {
  local name="$1"
  local spec="$2"
  local package="$3"
  local out_dir
  local jira_saved_dir=""

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

  # Save manually-maintained jira files before rm -rf wipes the directory
  if [[ "${name}" == "jira" && -d "${out_dir}" ]]; then
    jira_saved_dir="$(mktemp -d)"
    for _jf in src/models/attachment.rs \
               src/apis/agile_boards_api.rs src/apis/agile_sprints_api.rs \
               src/apis/users_api.rs src/apis/issue_worklogs_api.rs; do
      if [[ -f "${out_dir}/${_jf}" ]]; then
        mkdir -p "${jira_saved_dir}/$(dirname "${_jf}")"
        cp "${out_dir}/${_jf}" "${jira_saved_dir}/${_jf}"
      fi
    done
  fi

  rm -rf "${out_dir}"

  "${generator}" generate \
    -g rust \
    -i "${spec}" \
    -o "${out_dir}" \
    --additional-properties="packageName=${package},packageVersion=0.1.0,library=reqwest,reqwestDefaultFeatures=rustls-tls" \
    --global-property apis,models,supportingFiles

  fix_generated_manifest "${out_dir}/Cargo.toml"
  fix_generated_lints "${out_dir}/src/lib.rs"
  remove_standalone_scaffold "${out_dir}"
  fix_generated_docs "${out_dir}"

  # Restore and patch manually-maintained files for atla-jira-api
  if [[ "${name}" == "jira" ]]; then
    if [[ -n "${jira_saved_dir}" ]]; then
      for _jf in src/models/attachment.rs \
                 src/apis/agile_boards_api.rs src/apis/agile_sprints_api.rs \
                 src/apis/users_api.rs src/apis/issue_worklogs_api.rs; do
        if [[ -f "${jira_saved_dir}/${_jf}" ]]; then
          cp "${jira_saved_dir}/${_jf}" "${out_dir}/${_jf}"
        fi
      done
      rm -rf "${jira_saved_dir}"
    fi
    patch_jira_api_crate "${out_dir}"
  fi

  format_generated_crate "${out_dir}/Cargo.toml"
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
