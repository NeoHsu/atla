# OpenAPI Generator

Atla uses `openapi-generator-cli` for generated Jira and Confluence API client crates.

## Toolchain

- Generator: `openapi-generator-cli` `7.22.0`
- Java runtime: configured through `mise.toml`
- Wrapper: `scripts/openapi-generator.sh`
- Generation entrypoint: `scripts/generate.sh`

The generator jar is cached outside the repository under:

```text
~/.local/share/atla-tools/openapi-generator/
```

## Commands

```bash
scripts/openapi-generator.sh version
scripts/generate.sh --product jira
scripts/generate.sh --product confluence
scripts/generate.sh --product confluence-v1
```

By default, generated output goes to `target/openapi/` so it can be inspected before replacing workspace crates.

To intentionally refresh the generated crates:

```bash
scripts/generate.sh --product jira --in-place
scripts/generate.sh --product confluence --in-place
scripts/generate.sh --product confluence-v1 --in-place
```

## Current Finding

Full Jira Cloud v3 generation succeeds with openapi-generator `7.22.0`, but the resulting crate is very large:

- Around 1,000 generated files.
- Around 12 MB for Jira alone.
- A temporary `cargo check` of the full Jira generated crate exceeded 8 minutes on the current ARM container and one `rustc` process used around 1.8 GB RSS before the check was stopped.

Jira is currently generated as a scoped partial client under `crates/atla-jira-api` from `specs/jira-v3-partial.json`. The partial spec is built by `scripts/jira-v3-partial-spec.js` from Atlassian's v3 spec and currently includes:

- Project search and project view.
- JQL issue search and issue view.
- Issue create and issue update.
- Issue transitions and issue comments.

Confluence Cloud v2 generation is smaller and is now generated in-place under `crates/atla-confluence-api`:

- 31 generated API modules.
- 243 generated model modules.
- Around 550 generated files including generated endpoint/model docs and OpenAPI generator metadata.
- Around 3.3 MB.
- `cargo check --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` pass with the generated crate included.

Confluence Cloud v1 is generated as a scoped partial client under `crates/atla-confluence-v1-api` from `specs/confluence-v1-partial.json`. The partial spec is built by `scripts/confluence-v1-partial-spec.js` from Atlassian's v1 spec and currently includes:

- CQL search endpoints used by `atla confluence search`.
- Create/update attachment endpoint used by `atla confluence attachment upload`.

The v1 partial builder intentionally patches the generated-facing spec without changing the downloaded source spec: it removes a query parameter named `_` that generates invalid Rust, narrows the response models to the fields Atla consumes, and corrects attachment multipart fields so `minorEdit`, `comment`, and `X-Atlassian-Token: nocheck` generate as usable Rust client parameters.

The generated `Cargo.toml` also needs a small post-process fix: generator `7.22.0` emits `default = ["rustls-tls"]`, while the generated feature is named `rustls`. `scripts/generate.sh` applies that fix automatically.

Generated crates also run through post-processing that:

- Adds crate-level lint allowances for generator output that is valid Rust but not hand-written style.
- Runs `cargo fmt` for the generated crate.
- Removes standalone scaffold files that do not belong in this monorepo (`.gitignore`, `.travis.yml`, and `git_push.sh`).
- Normalizes generated Markdown docs by stripping trailing whitespace and extra final blank lines.

The full Confluence generated crate is expensive to build as a test artifact. On the current ARM container, `cargo test --workspace` spent more than 6 minutes compiling `atla-confluence-api` test artifacts with two `rustc` processes near 1 GB RSS each before being stopped. Use `cargo test --workspace --exclude atla-confluence-api` for the default hand-written crate test gate, while keeping generated client verification on `cargo check` and clippy.

For now, keep the generated crates as explicit workspace boundaries and avoid placing full generated output into default CI until we choose one of:

- Scoped generation by product/API group.
- Separate generated-client verification outside the default workspace gate.
- A custom template tuned for atla's workspace and dependency policy.
