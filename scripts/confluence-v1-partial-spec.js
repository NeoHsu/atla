#!/usr/bin/env node

const fs = require("fs");

const [, , inputPath, outputPath] = process.argv;

if (!inputPath || !outputPath) {
  console.error("Usage: scripts/confluence-v1-partial-spec.js INPUT OUTPUT");
  process.exit(2);
}

const spec = JSON.parse(fs.readFileSync(inputPath, "utf8"));
const selectedPaths = new Set([
  "/wiki/rest/api/content/search",
  "/wiki/rest/api/search",
  "/wiki/rest/api/search/user",
  "/wiki/rest/api/content/{id}/child/attachment",
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
    ["Content - attachments", "Search"].includes(tag.name),
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
    console.error(`missing selected path in source spec: ${path}`);
    process.exit(1);
  }
  partial.paths[path] = JSON.parse(JSON.stringify(spec.paths[path]));
}

for (const path of [
  "/wiki/rest/api/content/{id}/child/attachment",
  "/wiki/rest/api/content/{id}/child/attachment/{attachmentId}/data",
]) {
  if (!partial.paths[path]) continue;
  for (const operation of Object.values(partial.paths[path])) {
    normalizeAttachmentMultipartOperation(operation);
  }
}

for (const pathItem of Object.values(partial.paths)) {
  for (const operation of Object.values(pathItem)) {
    removeUnsupportedParameters(operation);
  }
}

Object.assign(partial.components.schemas, simplifiedSchemas());

function normalizeAttachmentMultipartOperation(operation) {
  if (!operation || typeof operation !== "object") return;

  operation.parameters ||= [];
  const hasXsrfHeader = operation.parameters.some(
    (parameter) =>
      parameter.name?.toLowerCase() === "x-atlassian-token" &&
      parameter.in === "header",
  );
  if (!hasXsrfHeader) {
    operation.parameters.push({
      name: "X-Atlassian-Token",
      in: "header",
      description:
        "Required by Confluence for attachment multipart requests. Use `nocheck`.",
      required: true,
      schema: {
        type: "string",
        default: "nocheck",
      },
    });
  }

  const schema =
    operation.requestBody?.content?.["multipart/form-data"]?.schema;
  if (!schema?.properties) return;

  if (schema.properties.comment) {
    schema.properties.comment = {
      ...schema.properties.comment,
      type: "string",
    };
    delete schema.properties.comment.format;
  }

  if (schema.properties.minorEdit) {
    schema.properties.minorEdit = {
      ...schema.properties.minorEdit,
      type: "boolean",
      default: false,
    };
    delete schema.properties.minorEdit.format;
  }
}

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
    console.error(`unsupported ref: ${ref}`);
    process.exit(1);
  }

  const source = spec.components?.[section]?.[name];
  if (partial.components?.[section]?.[name]) {
    collectRefs(partial.components[section][name]);
    continue;
  }
  if (!source) {
    console.error(`missing referenced component: ${ref}`);
    process.exit(1);
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
