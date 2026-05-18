#!/usr/bin/env node

const fs = require("fs");

const [, , inputPath, outputPath] = process.argv;

if (!inputPath || !outputPath) {
  console.error("Usage: scripts/jira-v3-partial-spec.js INPUT OUTPUT");
  process.exit(2);
}

const spec = JSON.parse(fs.readFileSync(inputPath, "utf8"));

const selectedOperations = {
  "/rest/api/3/issue": {
    post: {
      request: "IssueUpdateDetails",
      responses: {
        201: jsonResponse("CreatedIssue"),
      },
    },
  },
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
    put: {
      parameters: [
        pathParameter("issueIdOrKey", { type: "string" }),
      ],
      request: "IssueUpdateDetails",
      responses: {
        204: {
          description: "Returned if the request is successful.",
        },
      },
    },
  },
  "/rest/api/3/issue/{issueIdOrKey}/comment": {
    get: {
      parameters: [
        pathParameter("issueIdOrKey", { type: "string" }),
        queryParameter("startAt", { type: "integer", format: "int32" }),
        queryParameter("maxResults", { type: "integer", format: "int32" }),
      ],
      response: "PageOfComments",
    },
    post: {
      parameters: [
        pathParameter("issueIdOrKey", { type: "string" }),
      ],
      request: "CommentCreateRequest",
      responses: {
        201: jsonResponse("Comment"),
      },
    },
  },
  "/rest/api/3/issue/{issueIdOrKey}/transitions": {
    get: {
      parameters: [
        pathParameter("issueIdOrKey", { type: "string" }),
      ],
      response: "Transitions",
    },
    post: {
      parameters: [
        pathParameter("issueIdOrKey", { type: "string" }),
      ],
      request: "IssueTransitionRequest",
      responses: {
        204: {
          description: "Returned if the request is successful.",
        },
      },
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
      requestBody: config.request ? jsonRequest(config.request) : undefined,
      responses: config.responses || {
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

function jsonRequest(schemaName) {
  return {
    required: true,
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
    CreatedIssue: {
      type: "object",
      properties: {
        id: { type: "string" },
        key: { type: "string" },
        self: { type: "string" },
      },
    },
    IssueUpdateDetails: {
      type: "object",
      properties: {
        fields: {
          type: "object",
          additionalProperties: true,
        },
      },
    },
    IssueTransitionRequest: {
      type: "object",
      required: ["transition"],
      properties: {
        transition: {
          type: "object",
          required: ["id"],
          properties: {
            id: { type: "string" },
          },
        },
      },
    },
    Transitions: {
      type: "object",
      properties: {
        transitions: {
          type: "array",
          items: { $ref: "#/components/schemas/Transition" },
        },
      },
    },
    Transition: {
      type: "object",
      properties: {
        id: { type: "string" },
        name: { type: "string" },
        to: { $ref: "#/components/schemas/Status" },
      },
    },
    Status: {
      type: "object",
      properties: {
        id: { type: "string" },
        name: { type: "string" },
      },
    },
    CommentCreateRequest: {
      type: "object",
      required: ["body"],
      properties: {
        body: {
          type: "object",
          additionalProperties: true,
        },
      },
    },
    PageOfComments: {
      type: "object",
      properties: {
        startAt: { type: "integer", format: "int32" },
        maxResults: { type: "integer", format: "int32" },
        total: { type: "integer", format: "int32" },
        comments: {
          type: "array",
          items: { $ref: "#/components/schemas/Comment" },
        },
      },
    },
    Comment: {
      type: "object",
      properties: {
        id: { type: "string" },
        self: { type: "string" },
        body: {
          type: "object",
          additionalProperties: true,
        },
        author: { $ref: "#/components/schemas/User" },
        created: { type: "string" },
        updated: { type: "string" },
      },
    },
    User: {
      type: "object",
      properties: {
        accountId: { type: "string" },
        displayName: { type: "string" },
        active: { type: "boolean" },
      },
    },
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
