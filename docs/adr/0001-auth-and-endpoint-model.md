# ADR 0001: Authentication and endpoint model

- Status: Accepted
- Date: 2026-07-17

## Context

A single Atlassian site URL is sufficient for legacy, unscoped API tokens, but scoped API
tokens use product-specific gateway roots:

- `https://api.atlassian.com/ex/jira/{cloudId}`
- `https://api.atlassian.com/ex/confluence/{cloudId}`

The site URL is still required for browser links and profile identity. OAuth 2.0 (Atlassian
3LO) also requires a registered integration, redirect URI, authorization-code exchange, and
refresh-token lifecycle. A broadly distributed native binary cannot keep an embedded client
secret confidential.

## Decision

1. A profile retains `instance` as its canonical Atlassian site and credential identity.
2. An optional `cloud_id` activates scoped-token routing. Jira and Confluence derive separate
   gateway roots from that ID.
3. One API token is stored per profile for now. It remains outside `config.toml`, in the OS
   keyring or file credential store.
4. `atla auth discover --site URL` reads `/_edge/tenant_info` without credentials and returns
   the cloud ID and both gateway roots.
5. Legacy profiles without a cloud ID continue to use the site root without behavior changes.
6. OAuth 3LO is deferred. atla will not embed a client secret or label such a design secure.
   A future implementation must use an officially supported public-client flow or an optional
   broker that owns the confidential credential and documents its trust boundary.
7. Absolute signed download URLs may be requested, but Basic auth is attached only for the
   configured API origin. Cross-origin and HTTPS-to-HTTP redirects never receive credentials.

## OAuth spike conclusion

Atlassian's published 3LO documentation describes authorization-code grants for registered
integrations and rotating refresh tokens through `offline_access`. No official device
authorization flow is documented. Until Atlassian offers a suitable public native-client flow,
scoped API tokens are the secure first-class automation path for atla v1.

## Consequences

- Existing config remains compatible and migrates to schema version 2 with a backup.
- Scoped profiles work for both products without conflating their endpoint roots.
- Web links continue to use the human-facing site URL.
- Users must obtain and rotate tokens themselves and provide a cloud ID for scoped tokens.
- OAuth can be added later without changing the endpoint model.

## References

- [Atlassian API tokens](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/)
- [Atlassian OAuth 2.0 (3LO)](https://developer.atlassian.com/cloud/jira/platform/oauth-2-3lo-apps/)
- [Rotating refresh tokens](https://developer.atlassian.com/cloud/oauth/getting-started/refresh-tokens/)
