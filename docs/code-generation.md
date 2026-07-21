---
title: Code Generation
description: How atla generates Rust API clients from Atlassian OpenAPI specs using progenitor.
---

# Code generation

atla generates type-safe Rust API clients at compile time using
[progenitor](https://github.com/oxidecomputer/progenitor), a Rust-native OpenAPI code generator.
Normal Cargo builds consume checked-in partial specs and require no Java, Node.js, or Python
code-generation runtime. Refreshing those specs separately requires `curl`, Node.js, and Python 3.

---

## Architecture overview

```
Atlassian CDN (upstream specs)
        |
        v
  update-specs.sh           Download upstream OpenAPI JSON
        |
        v
  JS filter scripts          Filter/patch specs to partial subsets
        |
        v
  specs/*.json               Checked-in spec files
        |
        v
  build.rs (progenitor)      Generate Rust code at compile time
        |
        v
  $OUT_DIR/codegen.rs        Included via include!() in lib.rs
```

---

## Generated crates

| Crate | Spec file | Upstream source | Filter script |
|-------|-----------|-----------------|---------------|
| `atla-jira-api` | `specs/jira-v3-partial.json` | Jira Cloud v3 | `scripts/jira-v3-partial-spec.js` |
| `atla-confluence-api` | `specs/confluence-v2-partial.json` | Confluence Cloud v2 | `scripts/confluence-v2-partial-spec.js` |
| `atla-confluence-v1-api` | `specs/confluence-v1-partial.json` | Confluence Cloud v1 | `scripts/confluence-v1-partial-spec.js` |

Each crate contains only three hand-maintained files:

- `Cargo.toml` — declares build-dependencies on progenitor and runtime dependencies on progenitor-client
- `build.rs` — reads the spec and invokes progenitor to generate `$OUT_DIR/codegen.rs`
- `src/lib.rs` — `include!(concat!(env!("OUT_DIR"), "/codegen.rs"));`

All API client code is generated at compile time. There are no hand-maintained API modules.

---

## How build.rs works

All three crates follow the same pattern:

```rust
use progenitor::{Generator, GenerationSettings, InterfaceStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = "../../specs/<spec-file>.json";
    println!("cargo:rerun-if-changed={src}");

    let file = std::fs::File::open(src)?;
    let spec: openapiv3::OpenAPI = serde_json::from_reader(file)?;

    let mut settings = GenerationSettings::default();
    settings
        .with_interface(InterfaceStyle::Builder)
        .with_derive("PartialEq");

    let mut generator = Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec)
        .map_err(|error| std::io::Error::other(format!("generation failed: {error:?}")))?;
    let content = prettyplease::unparse(&syn::parse2(tokens)?);

    let out_path = std::path::Path::new(&std::env::var("OUT_DIR")?).join("codegen.rs");
    std::fs::write(out_path, content)?;
    Ok(())
}
```

Key points:

- `cargo:rerun-if-changed` ensures the crate only rebuilds when the spec file changes
- `InterfaceStyle::Builder` generates builder-pattern methods: `client.create_issue().body(body).send().await`
- `prettyplease` formats the generated code for readable compiler errors

### Build dependencies

```toml
[build-dependencies]
progenitor = "0.14"
serde_json = "1"
openapiv3 = "2"
syn = { version = "2", features = ["full"] }
prettyplease = "0.2"
```

### Runtime dependencies

```toml
[dependencies]
progenitor-client = "0.14"
reqwest.workspace = true
serde.workspace = true
serde_json.workspace = true
```

Some crates require additional dependencies (`chrono`, `uuid`, `serde_repr`,
etc.) depending on the spec's schema types.

---

## Local build cache and fast checks

Cargo already fingerprints each partial spec through `cargo:rerun-if-changed`,
so generated clients are reused until a spec or generator input changes. To
reuse that build output across worktrees and fresh checkouts, use the opt-in
shared target cache:

```bash
scripts/check-fast.sh
```

This runs `cargo check -p atla` and defaults `CARGO_TARGET_DIR` to
`$XDG_CACHE_HOME/atla/cargo-target` (or `~/.cache/atla/cargo-target`). Set
`ATLA_BUILD_CACHE_DIR` to choose another cache root, or set
`CARGO_TARGET_DIR` directly. Extra Cargo arguments are forwarded, for example
`scripts/check-fast.sh --all-targets`.

The cache never replaces release validation: run the full workspace fmt,
Clippy, and test commands before opening a PR. Delete the cache directory if
disk use or a suspected stale local artifact needs to be ruled out.

### Reference measurement

A local reference measurement on 2026-07-20 used `cargo check -p atla`, Rust
1.97.0, macOS arm64, and an Apple M2 Max:

| Target state | Wall time |
| --- | ---: |
| Empty isolated target directory | 35.66 s |
| Same target, no source changes | 0.60 s |

These numbers are a baseline, not a performance guarantee. Reproduce the
clean and warm checks with:

```bash
target_dir="$(mktemp -d)"
CARGO_TARGET_DIR="$target_dir" /usr/bin/time -p cargo check -p atla
CARGO_TARGET_DIR="$target_dir" /usr/bin/time -p cargo check -p atla
rm -rf "$target_dir"
```

---

## Spec filtering

### Why filter?

Full Atlassian specs are very large. The Jira v3 spec alone produces around 1,000 files and 12 MB of Rust code. Filtering to only the endpoints atla uses keeps compile times manageable.

### Jira partial spec (`scripts/jira-v3-partial-spec.js`)

Filters the full Jira Cloud v3 spec to include only:

- Issues: create, search, get, update, delete, transitions
- Comments: list, create, get, update, delete
- Projects: search, get
- Issue types, attachments, issue links

Also provides simplified schemas for complex types (`CreatedIssue`, `IssueUpdateDetails`,
`Transitions`, etc.) to avoid pulling in the entire Jira type graph. The simplified `Project`
schema deliberately keeps `projectTypeKey` open-ended because Atlassian returns values missing from
its published enum; the filter applies this invariant automatically.

Jira Software Agile endpoints (boards, sprints) are not part of the Jira platform v3 spec. atla calls those endpoints directly via raw `reqwest` calls in `atla-core`.

### Confluence v1 partial spec (`scripts/confluence-v1-partial-spec.js`)

Filters the Confluence v1 spec and applies compatibility patches:

- Paths included: content search, general search, user search, space info, and
  content labels.
- Unsupported query parameters are removed.
- Simplified schemas are provided for `Content`, `SearchResult`, `Space`, and
  related response models.

Attachment uploads are deliberately excluded from the generated client because
progenitor does not support `multipart/form-data`. atla sends them through its
raw reqwest multipart path while reusing the generated v1 response model. Page
label mutation also uses Confluence v1 raw REST calls because the v2 API does
not expose label add/remove endpoints.

### Confluence v2 partial spec (`scripts/confluence-v2-partial-spec.js`)

Filters the upstream v2 spec to the operations used by atla, then follows their
`$ref` closure so all required schemas remain available. The filter also
applies the documented enum and malformed-upstream-schema repairs from
`specs/PATCHES.md`. When core starts calling a new generated v2 operation, add
its snake_case operation name to `usedOperations` and refresh the specs.

---

## Updating specs

### Scheduled refresh

`.github/workflows/spec-refresh.yml` runs weekly and on manual dispatch. It
executes the update script, verifies fmt/check/workspace tests, and opens a
review PR containing only `specs/**` changes; it never pushes directly to
`main`. `scripts/spec-diff-summary.py` adds per-spec line/size/hash totals, operation-ID/schema-count
deltas, and normalized parameter/request/response/schema contract facts to the PR body and
workflow summary. This exposes nested field, requiredness, enum, type, and default changes that
counts alone miss. Review every invariant in `specs/PATCHES.md` and all contract diffs before
merging.

### Refresh workflow

The local refresh requires `curl`, Node.js, and Python 3 (`mise install` provisions the pinned
Node.js and Python versions). `scripts/update-specs.sh` handles the full refresh cycle:

1. Downloads upstream specs from Atlassian CDN
2. Runs JS filters that produce partial specs and apply every invariant in `specs/PATCHES.md`
3. Updates `specs/manifest.json` with SHA-256 hashes and metadata

```bash
scripts/update-specs.sh
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
cargo check --workspace
```

### Manifest (`specs/manifest.json`)

Tracks integrity and provenance for each spec:

- Source file path and SHA256 hash
- Upstream URL and SHA256 hash
- Filter script path (if applicable)
- Generator tool and version metadata

### Upstream spec URLs

| Spec | URL |
|------|-----|
| Jira v3 | `https://dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json` |
| Confluence v2 | `https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json` |
| Confluence v1 | `https://dac-static.atlassian.com/cloud/confluence/swagger.v3.json` |

---

## Core consumption pattern

progenitor generates a `Client` struct with builder-pattern methods. Core constructs it with the
shared `AtlassianClient::authed_http_client()` so credentials, timeouts, redirect protection, and
retry ownership stay centralized. Every generated call must then use the shared wrapper:

```rust
let client = atla_jira_api::Client::new_with_client(
    &base_url,
    raw_client.authed_http_client(),
);
let result = generated_request(reqwest::Method::POST, || {
    client.create_issue().body(body.clone()).send()
})
.await?;
let issue = result.into_inner();
```

Do not call a generated builder's `.send()` directly. `generated_request` applies bounded,
method-aware retry with exponential backoff and `Retry-After`, reads final API error bodies, retries
an explicit 429 rejection for any method, and otherwise never repeats a non-idempotent mutation.
Progenitor itself has no auth fields; Basic auth remains in the shared reqwest client.

---

## Adding new endpoints

To expose a new Jira or Confluence endpoint in atla:

1. **Identify the endpoint** in the upstream spec (check the downloaded spec in `specs/`)
2. **Add the operation** to the relevant JS filter script (for Confluence v2, add its snake_case
   operation name to `usedOperations`; the Jira/v1 scripts select paths)
3. **Re-run filtering**: `scripts/update-specs.sh`
4. **Verify**: `cargo check --workspace`
5. **Consume** the new generated method in `atla-core`

For endpoints not covered by any upstream spec (e.g. Jira Software Agile REST API), implement them directly in `atla-core` using raw `reqwest` calls.

---

## See also

- [Getting Started](./getting-started.md) — installation and first-time setup
- [Agent Reference](./agent-reference.md) — complete command reference for automation
