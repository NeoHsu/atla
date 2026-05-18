#!/usr/bin/env node

const fs = require("fs");

const [, , inputPath, outputPath] = process.argv;

if (!inputPath || !outputPath) {
  console.error("Usage: scripts/jira-v3-partial-spec.js INPUT OUTPUT");
  process.exit(2);
}

const spec = JSON.parse(fs.readFileSync(inputPath, "utf8"));

const selectedOperations = {
  "/rest/api/3/project/search": {
    get: {
      parameters: [
        queryParameter("startAt", { type: "integer", format: "int64" }),
        queryParameter("maxResults", { type: "integer", format: "int32" }),
        queryParameter("query", { type: "string" }),
      ],
      response: "PageBeanProject",
    },
  },
  "/rest/api/3/project/{projectIdOrKey}": {
    get: {
      parameters: [
        pathParameter("projectIdOrKey", { type: "string" }),
      ],
      response: "Project",
    },
  },
  "/rest/api/3/search/jql": {
    get: {
      parameters: [
        queryParameter("jql", { type: "string" }),
        queryParameter("nextPageToken", { type: "string" }),
        queryParameter("maxResults", { type: "integer", format: "int32" }),
        arrayQueryParameter("fields", { type: "string" }),
      ],
      response: "SearchAndReconcileResults",
    },
  },
  "/rest/api/3/issue/{issueIdOrKey}": {
    get: {
      parameters: [
        pathParameter("issueIdOrKey", { type: "string" }),
        arrayQueryParameter("fields", { type: "string" }),
      ],
      response: "IssueBean",
    },
  },
};

const partial = {
  openapi: spec.openapi,
  info: {
    ...spec.info,
    title: `${spec.info.title} - Atla partial`,
  },
  servers: spec.servers,
  security: spec.security,
  tags: (spec.tags || []).filter((tag) =>
    ["Issue search", "Issues", "Projects"].includes(tag.name),
  ),
  paths: {},
  components: {
    schemas: simplifiedSchemas(),
    securitySchemes: spec.components?.securitySchemes || {},
  },
};

for (const [path, methods] of Object.entries(selectedOperations)) {
  if (!spec.paths[path]) {
    console.error(`missing selected path in source spec: ${path}`);
    process.exit(1);
  }

  partial.paths[path] = {};

  for (const [method, config] of Object.entries(methods)) {
    const source = spec.paths[path][method];
    if (!source) {
      console.error(`missing selected operation in source spec: ${method.toUpperCase()} ${path}`);
      process.exit(1);
    }

    partial.paths[path][method] = {
      ...source,
      parameters: config.parameters,
      requestBody: undefined,
      responses: {
        200: jsonResponse(config.response),
      },
    };
  }
}

fs.writeFileSync(outputPath, `${JSON.stringify(partial, null, 2)}\n`);

function pathParameter(name, schema) {
  return {
    name,
    in: "path",
    required: true,
    schema,
  };
}

function queryParameter(name, schema) {
  return {
    name,
    in: "query",
    required: false,
    schema,
  };
}

function arrayQueryParameter(name, itemSchema) {
  return {
    name,
    in: "query",
    required: false,
    style: "form",
    explode: true,
    schema: {
      type: "array",
      items: itemSchema,
    },
  };
}

function jsonResponse(schemaName) {
  return {
    description: "OK",
    content: {
      "application/json": {
        schema: {
          $ref: `#/components/schemas/${schemaName}`,
        },
      },
    },
  };
}

function simplifiedSchemas() {
  return {
    PageBeanProject: {
      type: "object",
      properties: {
        isLast: { type: "boolean" },
        maxResults: { type: "integer", format: "int32" },
        startAt: { type: "integer", format: "int64" },
        total: { type: "integer", format: "int64" },
        values: {
          type: "array",
          items: { $ref: "#/components/schemas/Project" },
        },
      },
    },
    Project: {
      type: "object",
      properties: {
        id: { type: "string" },
        key: { type: "string" },
        name: { type: "string" },
        projectTypeKey: {
          type: "string",
          enum: ["software", "service_desk", "business"],
        },
        style: {
          type: "string",
          enum: ["classic", "next-gen"],
        },
        simplified: { type: "boolean" },
        archived: { type: "boolean" },
      },
    },
    SearchAndReconcileResults: {
      type: "object",
      properties: {
        isLast: { type: "boolean" },
        nextPageToken: { type: "string" },
        issues: {
          type: "array",
          items: { $ref: "#/components/schemas/IssueBean" },
        },
      },
    },
    IssueBean: {
      type: "object",
      properties: {
        id: { type: "string" },
        key: { type: "string" },
        fields: {
          type: "object",
          additionalProperties: true,
        },
      },
    },
  };
}
