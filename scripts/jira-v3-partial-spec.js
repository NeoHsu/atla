#!/usr/bin/env node

const fs = require("fs");

const [, , inputPath, outputPath] = process.argv;

function fail(message, exitCode = 1) {
	process.stderr.write(`${message}\n`);
	process.exit(exitCode);
}

if (!inputPath || !outputPath) {
	fail("Usage: scripts/jira-v3-partial-spec.js INPUT OUTPUT", 2);
}

let spec;
try {
	spec = JSON.parse(fs.readFileSync(inputPath, "utf8"));
} catch (error) {
	fail(`failed to read Jira v3 spec ${inputPath}: ${error.message}`);
}

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
			parameters: [pathParameter("projectIdOrKey", { type: "string" })],
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
			parameters: [pathParameter("issueIdOrKey", { type: "string" })],
			request: "IssueUpdateDetails",
			responses: {
				204: {
					description: "Returned if the request is successful.",
				},
			},
		},
		delete: {
			parameters: [
				pathParameter("issueIdOrKey", { type: "string" }),
				queryParameter("deleteSubtasks", { type: "boolean" }),
			],
			responses: {
				204: { description: "Returned if the request is successful." },
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
			parameters: [pathParameter("issueIdOrKey", { type: "string" })],
			request: "CommentCreateRequest",
			responses: {
				201: jsonResponse("Comment"),
			},
		},
	},
	"/rest/api/3/issue/{issueIdOrKey}/comment/{id}": {
		get: {
			parameters: [
				pathParameter("issueIdOrKey", { type: "string" }),
				pathParameter("id", { type: "string" }),
			],
			response: "Comment",
		},
		put: {
			parameters: [
				pathParameter("issueIdOrKey", { type: "string" }),
				pathParameter("id", { type: "string" }),
			],
			request: "CommentCreateRequest",
			responses: {
				200: jsonResponse("Comment"),
			},
		},
		delete: {
			parameters: [
				pathParameter("issueIdOrKey", { type: "string" }),
				pathParameter("id", { type: "string" }),
			],
			responses: {
				204: { description: "Returned if the request is successful." },
			},
		},
	},
	"/rest/api/3/issue/{issueIdOrKey}/transitions": {
		get: {
			parameters: [
				pathParameter("issueIdOrKey", { type: "string" }),
				queryParameter("expand", { type: "string" }),
			],
			response: "Transitions",
		},
		post: {
			parameters: [pathParameter("issueIdOrKey", { type: "string" })],
			request: "IssueTransitionRequest",
			responses: {
				204: {
					description: "Returned if the request is successful.",
				},
			},
		},
	},
	"/rest/api/3/issuetype/project": {
		get: {
			parameters: [queryParameter("projectId", { type: "string" })],
			response: "IssueTypeList",
		},
	},
	"/rest/api/3/issue/createmeta/{projectIdOrKey}/issuetypes/{issueTypeId}": {
		get: {
			parameters: [
				pathParameter("projectIdOrKey", { type: "string" }),
				pathParameter("issueTypeId", { type: "string" }),
				queryParameter("startAt", { type: "integer", format: "int32" }),
				queryParameter("maxResults", { type: "integer", format: "int32" }),
			],
			response: "IssueFieldPage",
		},
	},
	"/rest/api/3/attachment/{id}": {
		get: {
			parameters: [pathParameter("id", { type: "string" })],
			response: "Attachment",
		},
	},
	"/rest/api/3/issueLink": {
		post: {
			request: "LinkIssueRequestJsonBean",
			responses: {
				201: { description: "Returned if the request is successful." },
			},
		},
	},
	"/rest/api/3/issueLink/{linkId}": {
		delete: {
			parameters: [pathParameter("linkId", { type: "string" })],
			responses: {
				204: { description: "Returned if the request is successful." },
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
		[
			"Issue search",
			"Issues",
			"Projects",
			"Issue attachments",
			"Issue links",
			"Issue type schemes",
		].includes(tag.name),
	),
	paths: {},
	components: {
		schemas: simplifiedSchemas(),
		securitySchemes: spec.components?.securitySchemes || {},
	},
};

for (const [path, methods] of Object.entries(selectedOperations)) {
	if (!spec.paths[path]) {
		fail(`missing selected path in source spec: ${path}`);
	}

	partial.paths[path] = {};

	for (const [method, config] of Object.entries(methods)) {
		const source = spec.paths[path][method];
		if (!source) {
			fail(
				`missing selected operation in source spec: ${method.toUpperCase()} ${path}`,
			);
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

try {
	fs.writeFileSync(outputPath, `${JSON.stringify(partial, null, 2)}\n`);
} catch (error) {
	fail(`failed to write Jira v3 partial spec ${outputPath}: ${error.message}`);
}

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
				update: {
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
				fields: {
					type: "object",
					additionalProperties: true,
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
		IssueTypeList: {
			type: "array",
			items: { $ref: "#/components/schemas/IssueType" },
		},
		IssueType: {
			type: "object",
			properties: {
				id: { type: "string" },
				name: { type: "string" },
				description: { type: "string" },
				subtask: { type: "boolean" },
				iconUrl: { type: "string" },
			},
		},
		Attachment: {
			type: "object",
			properties: {
				id: { type: "string" },
				filename: { type: "string" },
				mimeType: { type: "string" },
				size: { type: "integer", format: "int64" },
				content: { type: "string" },
				thumbnail: { type: "string" },
				created: { type: "string" },
				author: { $ref: "#/components/schemas/User" },
			},
		},
		IssueFieldPage: {
			type: "object",
			properties: {
				startAt: { type: "integer", format: "int32" },
				maxResults: { type: "integer", format: "int32" },
				total: { type: "integer", format: "int32" },
				fields: {
					type: "array",
					items: { $ref: "#/components/schemas/IssueFieldMeta" },
				},
			},
		},
		IssueFieldMeta: {
			type: "object",
			properties: {
				fieldId: { type: "string" },
				key: { type: "string" },
				name: { type: "string" },
				required: { type: "boolean" },
				hasDefaultValue: { type: "boolean" },
				schema: {
					type: "object",
					additionalProperties: true,
				},
				allowedValues: {
					type: "array",
					items: {
						type: "object",
						additionalProperties: true,
					},
				},
				operations: {
					type: "array",
					items: { type: "string" },
				},
				autoCompleteUrl: { type: "string" },
			},
		},
		LinkIssueRequestJsonBean: {
			type: "object",
			properties: {
				type: {
					type: "object",
					properties: {
						name: { type: "string" },
					},
				},
				inwardIssue: {
					type: "object",
					properties: {
						key: { type: "string" },
					},
				},
				outwardIssue: {
					type: "object",
					properties: {
						key: { type: "string" },
					},
				},
			},
		},
	};
}
