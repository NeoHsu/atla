#!/usr/bin/env bash
set -euo pipefail

version="${OPENAPI_GENERATOR_VERSION:-7.22.0}"
tool_dir="${OPENAPI_GENERATOR_HOME:-${HOME}/.local/share/atla-tools/openapi-generator}"
jar="${tool_dir}/openapi-generator-cli-${version}.jar"
url="https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/${version}/openapi-generator-cli-${version}.jar"

if [[ ! -f "${jar}" ]]; then
  mkdir -p "${tool_dir}"
  curl -fL "${url}" -o "${jar}"
fi

if [[ -x "${HOME}/.local/bin/mise" ]]; then
  exec "${HOME}/.local/bin/mise" exec -- java -jar "${jar}" "$@"
fi

if command -v mise >/dev/null 2>&1; then
  exec "$(command -v mise)" exec -- java -jar "${jar}" "$@"
fi

exec java -jar "${jar}" "$@"
