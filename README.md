# atla

Unified Atlassian CLI for Jira and Confluence.

`atla` is planned as a Rust CLI with:

- Jira Cloud and Confluence Cloud support.
- API clients generated from Atlassian OpenAPI specs.
- Human-friendly table output and machine-friendly JSON/CSV/keys output.
- Profile-based authentication with tokens stored outside config files.

## Current Status

This repository is in Phase 1 setup:

- Cargo workspace skeleton.
- CLI command tree.
- Core module boundaries for auth, profile, client, and markdown conversion.
- Generated API crates reserved for OpenAPI generator output.

See `docs/project-atla/adr/init_atla_cli.md` in the planning workspace for the full ADR.

## Development

```bash
mise trust
mise install
cargo check
cargo test
```

