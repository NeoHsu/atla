# OpenAPI Specs

Generated API clients should be produced from official Atlassian specs:

- Jira Cloud v3: `https://dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json`
- Confluence Cloud v2: `https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json`

The Jira and Confluence v1 generated clients use checked-in partial specs for
the endpoint subsets currently wired into the CLI.

`specs/manifest.json` records the generator version, current spec sources, and
SHA256 checksums for downloaded and partial specs.

Run:

```bash
scripts/update-specs.sh
scripts/generate.sh
```
