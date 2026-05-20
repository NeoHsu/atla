# Spec Patches

This file documents intentional deviations from the upstream Jira/Confluence OpenAPI specs.
When updating a spec file, re-apply all patches listed here.

---

## `jira-v3-partial.json`

### 1. `Project.projectTypeKey` — remove enum constraint

**Location:** `#/components/schemas/Project/properties/projectTypeKey`

**Change:** Removed the `enum` array, keeping only `"type": "string"`.

**Reason:** Atlassian's upstream spec only lists `["software", "service_desk", "business"]` in the
`Project` response schema, but real instances return additional types such as `product_discovery`
(Jira Product Discovery / Polaris). Using a plain `String` prevents deserialization failures when
any unlisted project type appears in the API response.

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

## `confluence-v2.json`

### 1. `OnlyArchivedAndCurrentContentStatus` — remove enum constraint

**Location:** `#/components/schemas/OnlyArchivedAndCurrentContentStatus`

**Change:** Removed the `enum` array, keeping only `"type": "string"`.

**Reason:** Atlassian's upstream spec only lists `["current", "archived"]` but real instances
return additional statuses such as `"draft"` for child pages that are still in draft state.
Using a plain `String` prevents deserialization failures when any unlisted status appears
in the API response.

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
