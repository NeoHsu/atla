#!/usr/bin/env node

const fs = require("fs");

const [, , inputPath, outputPath] = process.argv;

function fail(message, exitCode = 1) {
  process.stderr.write(`${message}\n`);
  process.exit(exitCode);
}

if (!inputPath || !outputPath) {
  fail("Usage: scripts/confluence-v1-partial-spec.js INPUT OUTPUT", 2);
}

let spec;
try {
  spec = JSON.parse(fs.readFileSync(inputPath, "utf8"));
} catch (error) {
  fail(`failed to read Confluence v1 spec ${inputPath}: ${error.message}`);
}

// Attachment uploads use the raw reqwest multipart path. Progenitor does not
// support multipart/form-data request bodies, so keep that operation out of
// this model-only v1 client.
const selectedPaths = new Set([
  "/wiki/rest/api/content/search",
  "/wiki/rest/api/search",
  "/wiki/rest/api/search/user",
  "/wiki/rest/api/space/{spaceKey}",
  "/wiki/rest/api/content/{id}/label",
]);

const partial = {
  openapi: spec.openapi,
  info: {
    ...spec.info,
    title: `${spec.info.title} - Atla v1 partial`,
  },
  servers: spec.servers,
  security: spec.security,
  tags: (spec.tags || []).filter((tag) =>
    ["Content - attachments", "Search", "Space", "Content labels"].includes(
      tag.name,
    ),
  ),
  paths: {},
  components: {
    schemas: {},
    responses: {},
    parameters: {},
    examples: {},
    requestBodies: {},
    headers: {},
    securitySchemes: spec.components?.securitySchemes || {},
  },
};

for (const path of selectedPaths) {
  if (!spec.paths[path]) {
    fail(`missing selected path in source spec: ${path}`);
  }
  partial.paths[path] = structuredClone(spec.paths[path]);
}

for (const pathItem of Object.values(partial.paths)) {
  for (const operation of Object.values(pathItem)) {
    removeUnsupportedParameters(operation);
  }
}

Object.assign(partial.components.schemas, simplifiedSchemas());

function removeUnsupportedParameters(operation) {
  if (!operation?.parameters) return;
  operation.parameters = operation.parameters.filter(
    (parameter) => !(parameter.name === "_" && parameter.in === "query"),
  );
}

function simplifiedSchemas() {
  return {
    Content: {
      type: "object",
      required: ["type", "status"],
      properties: {
        id: { type: "string" },
        type: { type: "string" },
        status: { type: "string" },
        title: { type: "string" },
        version: { $ref: "#/components/schemas/Version" },
        _links: {
          type: "object",
          additionalProperties: { type: "string" },
        },
      },
    },
    ContentArray: {
      type: "object",
      required: ["results", "size"],
      properties: {
        results: {
          type: "array",
          items: { $ref: "#/components/schemas/Content" },
        },
        start: { type: "integer", format: "int32" },
        limit: { type: "integer", format: "int32" },
        size: { type: "integer", format: "int32" },
        _links: {
          type: "object",
          additionalProperties: { type: "string" },
        },
      },
    },
    SearchPageResponseSearchResult: {
      type: "object",
      required: ["results", "size", "totalSize"],
      properties: {
        results: {
          type: "array",
          items: { $ref: "#/components/schemas/SearchResult" },
        },
        start: { type: "integer", format: "int32" },
        limit: { type: "integer", format: "int32" },
        size: { type: "integer", format: "int32" },
        totalSize: { type: "integer", format: "int32" },
        cqlQuery: { type: "string" },
        searchDuration: { type: "integer", format: "int32" },
        _links: {
          type: "object",
          additionalProperties: { type: "string" },
        },
      },
    },
    SearchResult: {
      type: "object",
      required: ["title", "excerpt", "url"],
      properties: {
        title: { type: "string" },
        excerpt: { type: "string" },
        url: { type: "string" },
        content: { $ref: "#/components/schemas/Content" },
      },
    },
    Version: {
      type: "object",
      required: ["number"],
      properties: {
        when: { type: "string", format: "date-time" },
        message: { type: "string" },
        number: { type: "integer", format: "int32" },
        minorEdit: { type: "boolean" },
      },
    },
    Space: {
      type: "object",
      properties: {
        id: { type: "integer", format: "int64" },
        key: { type: "string" },
        name: { type: "string" },
        type: { type: "string" },
        status: { type: "string" },
        _links: {
          type: "object",
          additionalProperties: { type: "string" },
        },
      },
    },
  };
}

const seenRefs = new Set();
const queue = [];

function collectRefs(value) {
  if (!value || typeof value !== "object") return;
  if (typeof value.$ref === "string" && value.$ref.startsWith("#/components/")) {
    queue.push(value.$ref);
  }
  if (Array.isArray(value)) {
    for (const item of value) collectRefs(item);
    return;
  }
  for (const child of Object.values(value)) collectRefs(child);
}

collectRefs(partial.paths);

while (queue.length > 0) {
  const ref = queue.shift();
  if (seenRefs.has(ref)) continue;
  seenRefs.add(ref);

  const [, , section, name] = ref.split("/");
  if (!section || !name) {
    fail(`unsupported ref: ${ref}`);
  }

  const source = spec.components?.[section]?.[name];
  if (partial.components?.[section]?.[name]) {
    collectRefs(partial.components[section][name]);
    continue;
  }
  if (!source) {
    fail(`missing referenced component: ${ref}`);
  }

  partial.components[section] ||= {};
  partial.components[section][name] = source;
  collectRefs(source);
}

for (const section of Object.keys(partial.components)) {
  if (
    section !== "securitySchemes" &&
    Object.keys(partial.components[section]).length === 0
  ) {
    delete partial.components[section];
  }
}

fs.writeFileSync(outputPath, `${JSON.stringify(partial, null, 2)}\n`);
