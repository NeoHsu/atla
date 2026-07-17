# Spec Patches

This file documents intentional deviations from the upstream Jira and Confluence
OpenAPI specs. When updating a spec file, re-apply all patches listed here.

---

## `jira-v3-partial.json`

### 1. `Project.projectTypeKey` — remove enum constraint

**Location:** `#/components/schemas/Project/properties/projectTypeKey`

**Change:** Removed the `enum` array, keeping only `"type": "string"`.

**Reason:** Atlassian's upstream spec only lists
`["software", "service_desk", "business"]` in the `Project` response schema,
but real instances return additional types such as `product_discovery` (Jira
Product Discovery / Polaris). Using a plain `String` prevents deserialization
failures when any unlisted project type appears in the API response.

**Original (upstream):**

```json
"projectTypeKey": {
  "type": "string",
  "enum": ["software", "service_desk", "business"]
}
```

**Patched:**

```json
"projectTypeKey": {
  "type": "string"
}
```

---

## `confluence-v1-partial.json`

### 1. Attachment multipart operations — exclude from generated client

**Applied automatically** by `scripts/confluence-v1-partial-spec.js`.

**Location:** `/wiki/rest/api/content/{id}/child/attachment`

**Change:** Do not copy the upload operations into the partial generated-client
spec.

**Reason:** Progenitor rejects `multipart/form-data` request bodies. atla
already sends attachment uploads through its raw reqwest multipart path and
uses only its generated v1 response model. Including these unused operations
would break code generation without adding runtime coverage.

---

## `confluence-v2-partial.json`

### 1. `OnlyArchivedAndCurrentContentStatus` — remove enum constraint

**Applied automatically** by `scripts/confluence-v2-partial-spec.js` (see its
`stripEnumSchemas` set) — no manual re-application needed on spec refresh.

**Location:** `#/components/schemas/OnlyArchivedAndCurrentContentStatus`

**Change:** Removed the `enum` array, keeping only `"type": "string"`.

**Reason:** Atlassian's upstream spec only lists `["current", "archived"]`, but
real instances return additional statuses such as `"draft"` for child pages
that are still in draft state. Using a plain `String` prevents deserialization
failures when any unlisted status appears in the API response.

**Original (upstream):**

```json
"OnlyArchivedAndCurrentContentStatus": {
  "enum": ["current", "archived"],
  "type": "string",
  "description": "The status of the content."
}
```

**Patched:**

```json
"OnlyArchivedAndCurrentContentStatus": {
  "type": "string"
}
```

### 2. `PageUpdateRequest` scalar IDs — repair invalid `format: string`

**Applied automatically** by `scripts/confluence-v2-partial-spec.js`.

**Location:** Start at
`#/components/requestBodies/PageUpdateRequest`, then follow
`content/application~1json/schema/properties`. This applies to `spaceId`,
`parentId`, and `ownerId`.

**Change:** Replace a scalar schema containing `"format": "string"` without a
`type` with `"type": "string"`.

**Reason:** The upstream spec published on 2026-07-17 used `string` as a format
rather than a JSON Schema type. `typify` rejects that schema, so progenitor
cannot generate the client.

### 3. Scalar `sort` parameters — remove array-only `items` wrapper

**Applied automatically** by `scripts/confluence-v2-partial-spec.js`.

**Location:** Selected v2 operations whose scalar `sort` query parameter is
backed by `LabelSortOrder` or `ContentSortOrder`.

**Change:** Replace `{"type":"string","items":{"$ref":"..."}}` with the
previous direct scalar `{"$ref":"..."}`.

**Reason:** `items` is valid only for array schemas. Keeping the direct enum
reference preserves both the API's scalar query contract and the generated
method signature.
