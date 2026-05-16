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
```

By default, generated output goes to `target/openapi/` so it can be inspected before replacing workspace crates.

To intentionally refresh the generated crates:

```bash
scripts/generate.sh --product jira --in-place
scripts/generate.sh --product confluence --in-place
```

## Current Finding

Full Jira Cloud v3 generation succeeds with openapi-generator `7.22.0`, but the resulting crate is very large:

- Around 1,000 generated files.
- Around 12 MB for Jira alone.
- A temporary `cargo check` of the full Jira generated crate exceeded 8 minutes on the current ARM container and one `rustc` process used around 1.8 GB RSS before the check was stopped.

Confluence Cloud v2 generation is smaller and passes a temporary crate check:

- Around 280 generated files.
- `cargo check --manifest-path /tmp/atla-gen-script-test/confluence/Cargo.toml` finished in about 1.5 minutes.
- The generated crate currently emits Rust naming warnings for a few model modules, but no compile errors.

The generated `Cargo.toml` also needs a small post-process fix: generator `7.22.0` emits `default = ["rustls-tls"]`, while the generated feature is named `rustls`. `scripts/generate.sh` applies that fix automatically.

For now, keep the generated crates as explicit workspace boundaries and avoid placing full generated output into default CI until we choose one of:

- Scoped generation by product/API group.
- Separate generated-client verification outside the default workspace gate.
- A custom template tuned for atla's workspace and dependency policy.
