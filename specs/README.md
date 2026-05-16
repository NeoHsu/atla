# OpenAPI Specs

Generated API clients should be produced from official Atlassian specs:

- Jira Cloud v3: `https://dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json`
- Confluence Cloud v2: `https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json`

Run:

```bash
scripts/update-specs.sh
scripts/generate.sh
```

