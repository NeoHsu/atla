# OpenAPI Specs

All generated API clients are built from checked-in partial specs derived from official Atlassian
sources:

- Jira Cloud v3: `https://dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json`
- Confluence Cloud v2: `https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json`
- Confluence Cloud v1: `https://dac-static.atlassian.com/cloud/confluence/swagger.v3.json`

`specs/manifest.json` records the generator version, upstream and partial source paths, source URLs,
and SHA-256 checksums. Normal Cargo builds consume only the checked-in partial specs.

To refresh specs, install `curl`, Node.js, and Python 3, then run:

```bash
scripts/update-specs.sh   # download upstream specs, rebuild partial specs, update manifest
cargo build               # regenerate each API crate's client through build.rs
```

The Jira, Confluence v1, and Confluence v2 filter scripts apply every invariant documented in
`specs/PATCHES.md` automatically. After refreshing, review those invariants and verify that every
filter exactly reproduces its checked-in partial spec and all manifest hashes match:

```bash
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```
