# Migration Plan: openapi-generator to progenitor

Replace the Java-based openapi-generator-cli with Rust-native [progenitor](https://github.com/oxidecomputer/progenitor) for API client code generation.

## Motivation

Current pipeline has significant maintenance overhead:

- **7-step post-processing**: generate -> fix Cargo.toml -> fix lint -> patch functions -> save/restore manual files -> remove scaffold -> format
- **3 external tools**: Java runtime, openapi-generator JAR, Python (for regex patching)
- **5 hand-maintained files** inside generated crates (save/restore on every regeneration)
- **Complex CI**: `check-generated` job reverse-engineers which files are manual vs generated
- **Fragile patches**: Python heredocs embedded in shell scripts inject entire functions into generated code

With progenitor (build.rs mode), the pipeline becomes:

```
upstream spec -> JS filter/patch -> cargo build (build.rs generates at compile time)
```

No external tools. No post-processing. No CI check-generated.

---

## Current State

### Generated Crates

| Crate | Spec Source | Notes |
|-------|-----------|-------|
| `atla-jira-api` | `specs/jira-v3-partial.json` | 5 hand-maintained files, 4 patched functions |
| `atla-confluence-api` | `specs/confluence-v2.json` | Clean, no manual code |
| `atla-confluence-v1-api` | `specs/confluence-v1-partial.json` | Clean, no manual code |

### Hand-Maintained Files (atla-jira-api only)

| File | Purpose |
|------|---------|
| `src/models/attachment.rs` | Custom deserializer: Jira returns `id` as int or string |
| `src/apis/agile_boards_api.rs` | Agile board endpoints (not in Jira v3 spec) |
| `src/apis/agile_sprints_api.rs` | Agile sprint endpoints (not in Jira v3 spec) |
| `src/apis/users_api.rs` | `/myself`, `/user/assignable/search` |
| `src/apis/issue_worklogs_api.rs` | Worklog add/list |

### Patched Functions (injected by generate.sh)

| Patch | Target |
|-------|--------|
| `remove_attachment` | `issue_attachments_api.rs` |
| `set_assignee` | `issues_api.rs` |
| `project_type_key` -> `Option<String>` | `project.rs` |
| Module declarations for hand-maintained modules | `apis/mod.rs` |

### Core Consumption Pattern

```rust
// Current: openapi-generator style (free functions + Configuration)
let cfg = generated_apis::configuration::Configuration { base_path, client, basic_auth, .. };
generated_apis::issues_api::create_issue(&cfg, body).await
```

```rust
// Target: progenitor style (Client struct + builder methods)
let client = atla_jira_api::Client::new_with_client(&base_url, reqwest_client);
client.create_issue().body(body).send().await
```

---

## Phase 0: Feasibility Validation

**Goal**: Confirm progenitor can parse all 3 specs and identify required spec fixes. No production code changes.

> **Spike results** (validated against Progenitor 0.14):
> - `jira-v3-partial.json` ✅ compiles successfully
> - `confluence-v2.json` ❌ fails: `components/requestBodies/PageUpdateRequest/.../ownerId` has `format: "string"` but no `type` — fixable in JS filter
> - `confluence-v1-partial.json` ❌ fails: `UnexpectedFormat("unexpected content type: multipart/form-data")` — progenitor does not support multipart; attachment upload endpoints must remain manually maintained in Phase 3

### Steps

1. Create a temporary test crate outside the workspace with the following build-dependencies:
   ```toml
   [build-dependencies]
   progenitor = "0.14"
   serde_json = "1"
   openapiv3 = "2"
   syn = { version = "2", features = ["full"] }
   prettyplease = "0.2"
   ```
2. Add `build.rs` that feeds each spec to `progenitor::Generator`
3. Run `cargo check` for each spec individually
4. Catalog all errors — classify as spec issues vs progenitor limitations
5. For spec issues: prototype fixes in the JS filtering scripts
6. Document old-to-new type name mapping (progenitor naming conventions differ)

### Exit Criteria

- Jira partial spec produces compilable Rust code (already confirmed)
- Confluence v2 schema issue fixed in JS filter and confirmed compilable
- Confluence v1 multipart endpoints catalogued and excluded from generation scope
- A type name mapping document exists
- JS script fixes are identified but not yet committed

---

## Phase 0.5: Upgrade reqwest workspace-wide

**Required before any crate migration.** Progenitor 0.12+ depends on `reqwest 0.13`; the workspace currently uses `reqwest 0.12`.

1. Bump `reqwest` in `Cargo.toml` workspace dependencies: `"0.12"` → `"0.13"`
2. Fix any breaking API changes in `atla-core` and `atla-cli` (reqwest 0.12→0.13 has minimal surface changes but verify multipart and stream usage)
3. Confirm `cargo check --workspace` passes before proceeding

---

## Phase 1: Migrate atla-jira-api

The most complex crate. Has hand-maintained files and patches.

### Step 1.1: Set Up progenitor build.rs

Create `crates/atla-jira-api/build.rs`:

```rust
use progenitor::{Generator, GenerationSettings, InterfaceStyle};

fn main() {
    let src = "../../specs/jira-v3-partial.json";
    println!("cargo:rerun-if-changed={}", src);

    let file = std::fs::File::open(src).unwrap();
    // Must deserialize as openapiv3::OpenAPI, not serde_json::Value
    let spec: openapiv3::OpenAPI = serde_json::from_reader(file).unwrap();

    let mut settings = GenerationSettings::default();
    settings
        .with_interface(InterfaceStyle::Builder)
        .with_derive("PartialEq");

    let mut generator = Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();
    let content = prettyplease::unparse(&ast);

    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("codegen.rs");
    std::fs::write(out, content).unwrap();
}
```

Update `lib.rs`:

```rust
include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
```

Write `Cargo.toml` manually (no longer generator-produced). Build-dependencies required:

```toml
[build-dependencies]
progenitor = "0.14"
serde_json = "1"
openapiv3 = "2"
syn = { version = "2", features = ["full"] }
prettyplease = "0.2"

[dependencies]
progenitor-client = "0.14"
reqwest.workspace = true
serde.workspace = true
serde_json.workspace = true
```

### Step 1.2: Eliminate Hand-Maintained Files

Add missing endpoints to `scripts/jira-v3-partial-spec.js` so progenitor generates them:

| Hand-maintained file | Resolution |
|---------------------|---------------------|
| `agile_boards_api.rs` | ⚠️ Agile paths are **not in `jira-v3.json`** — must merge from a second upstream spec (Jira Agile REST API) or continue to hand-maintain |
| `agile_sprints_api.rs` | Same as above |
| `users_api.rs` | Add `/rest/api/3/myself`, `/rest/api/3/user/assignable/search` to partial spec |
| `issue_worklogs_api.rs` | Add `/rest/api/3/issue/{issueIdOrKey}/worklog` to partial spec |
| `attachment.rs` deserializer | **Cannot use `TypePatch`** — it only renames/adds derives, cannot change per-field deserialization. `TypePatch` and setting `id` to `type: string` are both insufficient because Jira sometimes returns a bare integer, which will fail `String` deserialization. **Solution**: delete `attachment.rs` from the generated crate and add an `AttachmentId` newtype in `atla-core` with a custom `#[serde(deserialize_with)]` deserializer. `From<generated::Attachment>` conversions use the newtype. |

### Step 1.3: Eliminate Patched Functions

| Patch | Resolution |
|-------|-----------|
| `remove_attachment` | Add `DELETE /rest/api/3/attachment/{id}` to partial spec |
| `set_assignee` | Add `PUT /rest/api/3/issue/{issueIdOrKey}/assignee` to partial spec |
| `project_type_key` -> `Option<String>` | Set `type: string` in spec schema (drop enum) |
| Module declarations in `mod.rs` | No longer needed (progenitor generates a single file) |

### Step 1.4: Update Core Jira Consumption

Key files to change (~35–39 call sites):

| File | Call sites | Notes |
|------|-----------|-------|
| `crates/atla-core/src/jira/mod.rs` | Client construction | `Configuration` -> progenitor `Client` |
| `crates/atla-core/src/jira/issues.rs` | 10 | Largest consumer |
| `crates/atla-core/src/jira/util.rs` | Error mapping | `generated_error()` rewrite — see below |
| `crates/atla-core/src/jira/comments.rs` | 4 | |
| `crates/atla-core/src/jira/projects.rs` | 3 | |
| `crates/atla-core/src/jira/sprints.rs` | 7 | |
| `crates/atla-core/src/jira/boards.rs` | 2 | |
| Other jira modules | ~3 | attachments, links, types |

**Auth construction**: Progenitor's `Client` has no `basic_auth` field. Build a pre-configured `reqwest::Client` with `Authorization` default header:

```rust
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
let mut headers = HeaderMap::new();
let creds = base64::encode(format!("{}:{}", email, api_token));
headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", creds))?);
let http_client = reqwest::Client::builder().default_headers(headers).build()?;
let client = atla_jira_api::Client::new_with_client(&base_url, http_client);
```

**Error model migration**: `generated_error()` must become `async fn` because `UnexpectedResponse(reqwest::Response)` requires `.text().await` to extract the body. This cascades to all callers — every `.map_err(generated_error)` chain must be replaced with `.await.map_err(|e| generated_error(e).await)` or equivalent.

```rust
// progenitor error model
pub enum Error<E = ()> {
    InvalidRequest(String),              // builder validation failure — map to ApiError::Decode
    CommunicationError(reqwest::Error),  // map to ApiError::Decode
    ErrorResponse(ResponseValue<E>),     // map to ApiError::Http { status, body }
    InvalidResponsePayload(bytes::Bytes, reqwest::Error), // map to ApiError::Decode
    UnexpectedResponse(reqwest::Response), // requires .text().await — async only
}
```

`.send().await` returns `Result<ResponseValue<T>, Error<E>>`; unwrap the value with `.into_inner()` to get `T`.

### Step 1.5: Update `From` Impls

All `From<generated_models::X>` impls in core need updating for new type names and potentially different field types (progenitor may use `chrono::DateTime` instead of `String` for dates, etc.).

---

## Phase 2: Migrate atla-confluence-api

**Simpler than Phase 1** — no hand-maintained files or patches.

1. Fix `confluence-v2.json` schema in `scripts/confluence-v2-spec.js` (or inline patch): add `"type": "string"` to the `ownerId` property in `PageUpdateRequest` (currently has `format` without `type`, causing a progenitor parse error)
2. Add `build.rs` reading `specs/confluence-v2.json` (same structure as Step 1.1)
3. Write `Cargo.toml` manually
4. Update `crates/atla-core/src/confluence/` call sites (~26)
5. Update `From` impls for Confluence models

---

## Phase 3: Migrate atla-confluence-v1-api

**Smallest scope**, but with a known limitation.

> ⚠️ **Progenitor does not support `multipart/form-data`**. The attachment upload endpoint in `confluence-v1-partial.json` cannot be generated. It must remain as a manually-maintained method.

1. Add `build.rs` reading `specs/confluence-v1-partial.json`
2. Write `Cargo.toml` manually
3. Update core call sites (~6)
4. `ConfluenceClient` holds both v2 and v1 clients; update both fields
5. Manually retain the attachment upload endpoint outside the generated include (e.g., a separate `upload.rs` in the crate or inline in `atla-core`)

---

## Phase 4: Cleanup

### Files to Delete

```
scripts/generate.sh
scripts/openapi-generator.sh
crates/atla-*-api/.openapi-generator/         (entire directories)
crates/atla-*-api/.openapi-generator-ignore
crates/atla-*-api/docs/                        (generated markdown docs)
```

### CI Changes

- Remove `check-generated` job from `.github/workflows/ci.yml`
- Remove generate step from `.github/workflows/update-specs.yml` (keep spec fetching)
- Remove Java runtime requirement from CI
- Add `cargo:rerun-if-changed` ensures specs trigger rebuild automatically

### Dependency Cleanup

- Remove `serde_repr` and `url` from workspace if no longer needed
- Add `progenitor-client` to workspace dependencies

### Documentation Updates

- Update `docs/getting-started.md`
- Update this migration plan to mark completion

---

## Risk Mitigation

### Incremental Migration

Each phase is an independent PR. The project can run with a mix of old and new crates during transition. If progenitor proves unworkable for one spec, that crate stays on the old system.

### Rollback Path

Old generated code is in git history. Any phase can be reverted independently.

### Testing Strategy

- **Generator-agnostic tests**: JSON deserialization tests in core test domain models directly — no changes needed
- **Generator-dependent tests**: `From<generated_models::X>` tests must be updated per phase
- **CI validation**: `cargo check --workspace` + existing test suite must pass at each phase

### Known Risks

| Risk | Impact | Mitigation |
|------|--------|-----------|
| progenitor rejects a spec | Blocks that phase | Fix in JS filter script; fall back to openapi-generator for that crate |
| **reqwest version conflict** (workspace on 0.12, progenitor 0.12+ requires 0.13) | **Blocks all phases** | Upgrade reqwest workspace-wide in Phase 0.5 before any crate migration |
| **`multipart/form-data` not supported by progenitor** | Confluence v1 attachment upload cannot be generated | Manually maintain upload endpoint; exclude from generation scope |
| **`generated_error()` must become `async fn`** | Cascades to all `.map_err(generated_error)` call sites | Rewrite as async in Phase 1.4; all callers in `atla-core/jira` and `atla-core/confluence` must be updated |
| Type name changes break `From` impls | Compile errors in core | Phase 0 produces a mapping document |
| progenitor's auth model differs | `Client` has no `basic_auth` field | Construct `reqwest::Client` with `default_headers` containing the `Authorization` header |
| Build time increase (spec parsed every build) | Slower dev cycle | `cargo:rerun-if-changed` limits rebuilds to spec changes only |
| Confluence v2 full spec schema errors | Fails progenitor parse | Fix `ownerId` property in JS filter (add `"type": "string"`); create `confluence-v2-partial-spec.js` if more issues found |
| Agile endpoints absent from all Jira v3 specs | Agile board/sprint files may need to remain manual | Merge from Jira Agile REST API spec or accept continued manual maintenance |

---

## Summary

| Phase | Scope | Complexity | Est. Files Changed |
|-------|-------|-----------|-------------------|
| 0 - Validation | Exploratory, no production changes | Medium | 0 |
| 0.5 - reqwest upgrade | Workspace-wide dependency bump | Low | ~3 |
| 1 - Jira | Most hand-maintained code and patches; async error rewrite | High | ~20–25 |
| 2 - Confluence v2 | Schema fix + no manual code | Medium | ~12 |
| 3 - Confluence v1 | Smallest crate; multipart endpoint stays manual | Low | ~6 |
| 4 - Cleanup | Delete old pipeline | Low | ~8 |

**Estimated total effort: 4–5 engineer-weeks.**

**Start with Phase 0** to confirm the Confluence v2 schema fix compiles, then Phase 0.5 (reqwest upgrade), before committing to the full migration.

> **Phase ordering note**: Consider migrating in order of increasing complexity — Confluence v1 → Confluence v2 → Jira — to build familiarity with progenitor patterns before tackling the most complex crate. The plan above follows original phase numbering; reorder if preferred.
