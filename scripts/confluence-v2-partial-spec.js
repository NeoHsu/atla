#!/usr/bin/env node
// Filters the full Confluence v2 OpenAPI spec down to the operations atla-core
// actually calls, keeping the transitive $ref closure of their schemas intact
// so the generated types stay byte-for-byte compatible with the full spec.
//
// Unlike scripts/jira-v3-partial-spec.js (which hand-rebuilds a minimal spec),
// this script prunes the upstream document, so refreshing the spec cannot
// drift from the real schemas.
//
// To regenerate the operation list:
//   cd crates/atla-core/src/confluence && cat *.rs | tr -d ' \n' \
//     | grep -oE '\.generated\.[a-z_0-9]+\(' | sed 's/\.generated\.//;s/(//' | sort -u

const fs = require("fs");

const [, , inputPath, outputPath] = process.argv;
function fail(message, exitCode = 1) {
	process.stderr.write(`${message}\n`);
	process.exit(exitCode);
}

if (!inputPath || !outputPath) {
	fail("Usage: scripts/confluence-v2-partial-spec.js INPUT OUTPUT", 2);
}

// snake_case progenitor method names for every v2 operation atla-core calls.
const usedOperations = new Set([
	"create_blog_post",
	"create_footer_comment",
	"create_page",
	"create_space",
	"delete_attachment",
	"delete_blog_post",
	"delete_footer_comment",
	"delete_page",
	"get_attachment_by_id",
	"get_blog_post_by_id",
	"get_blog_post_footer_comments",
	"get_blog_post_labels",
	"get_blog_posts",
	"get_page_attachments",
	"get_page_by_id",
	"get_page_descendants",
	"get_page_direct_children",
	"get_page_footer_comments",
	"get_page_labels",
	"get_pages",
	"get_spaces",
	"update_blog_post",
	"update_page",
	"update_page_title",
]);

// Schemas whose enum constraint must be stripped: upstream enums lag behind
// values real instances return (see specs/PATCHES.md).
const stripEnumSchemas = new Set(["OnlyArchivedAndCurrentContentStatus"]);

// progenitor derives method names by snake_casing the operationId.
function snakeCase(operationId) {
	return operationId
		.replace(/([a-z0-9])([A-Z])/g, "$1_$2")
		.replace(/[-\s]+/g, "_")
		.toLowerCase();
}

let spec;
try {
	spec = JSON.parse(fs.readFileSync(inputPath, "utf8"));
} catch (error) {
	fail(`failed to read Confluence v2 spec ${inputPath}: ${error.message}`);
}

const HTTP_METHODS = [
	"get",
	"put",
	"post",
	"delete",
	"options",
	"head",
	"patch",
	"trace",
];
normalizeKnownUpstreamDefects(spec);

// Atlassian occasionally publishes scalar schemas as `format: string` with no
// type, or as `type: string` plus an array-only `items` reference. Both forms
// are invalid for the scalar fields involved and break or drift progenitor.
function normalizeKnownUpstreamDefects(document) {
	const pageUpdateProperties =
		document.components?.requestBodies?.PageUpdateRequest?.content?.[
			"application/json"
		]?.schema?.properties;
	for (const name of ["spaceId", "parentId", "ownerId"]) {
		const schema = pageUpdateProperties?.[name];
		if (schema?.format === "string" && schema.type === undefined) {
			schema.type = "string";
			delete schema.format;
		}
	}

	for (const pathItem of Object.values(document.paths ?? {})) {
		for (const method of HTTP_METHODS) {
			const operation = pathItem[method];
			for (const parameter of operation?.parameters ?? []) {
				const schema = parameter?.schema;
				if (
					parameter?.name === "sort" &&
					schema?.type === "string" &&
					schema.items?.$ref
				) {
					parameter.schema = { $ref: schema.items.$ref };
				}
			}
		}
	}
}

const keptPaths = {};
const seenOperations = new Set();
for (const [path, item] of Object.entries(spec.paths ?? {})) {
	const kept = {};
	for (const [key, value] of Object.entries(item)) {
		if (!HTTP_METHODS.includes(key)) {
			kept[key] = value; // path-level parameters, servers, etc.
			continue;
		}
		const name = value.operationId ? snakeCase(value.operationId) : null;
		if (name && usedOperations.has(name)) {
			kept[key] = value;
			seenOperations.add(name);
		}
	}
	if (HTTP_METHODS.some((m) => m in kept)) {
		keptPaths[path] = kept;
	}
}

const missing = [...usedOperations].filter((op) => !seenOperations.has(op));
if (missing.length > 0) {
	fail(`operations not found in ${inputPath}: ${missing.join(", ")}`);
}

// Transitive $ref closure starting from the kept paths.
const componentsIn = spec.components ?? {};
const keptComponents = {};
const queue = [];

function collectRefs(node) {
	if (Array.isArray(node)) {
		node.forEach(collectRefs);
		return;
	}
	if (node && typeof node === "object") {
		for (const [key, value] of Object.entries(node)) {
			if (key === "$ref" && typeof value === "string") {
				const match = value.match(/^#\/components\/([^/]+)\/(.+)$/);
				if (match) queue.push([match[1], match[2]]);
			} else {
				collectRefs(value);
			}
		}
	}
}

collectRefs(keptPaths);
while (queue.length > 0) {
	const [section, name] = queue.pop();
	if (keptComponents[section]?.[name] !== undefined) continue;
	const definition = componentsIn[section]?.[name];
	if (definition === undefined) {
		fail(`unresolved reference: #/components/${section}/${name}`);
	}
	(keptComponents[section] ??= {})[name] = definition;
	collectRefs(definition);
}

if (componentsIn.securitySchemes) {
	keptComponents.securitySchemes = componentsIn.securitySchemes;
}

for (const name of stripEnumSchemas) {
	const schema = keptComponents.schemas?.[name];
	if (schema && "enum" in schema) {
		delete schema.enum;
	}
}

// Deterministic component ordering keeps diffs reviewable.
const sortedComponents = {};
for (const section of Object.keys(keptComponents).sort()) {
	sortedComponents[section] = {};
	for (const name of Object.keys(keptComponents[section]).sort()) {
		sortedComponents[section][name] = keptComponents[section][name];
	}
}

const output = {
	openapi: spec.openapi,
	info: spec.info,
	servers: spec.servers,
	paths: keptPaths,
	components: sortedComponents,
};
if (spec.security) output.security = spec.security;

fs.writeFileSync(outputPath, `${JSON.stringify(output, null, 2)}\n`);
const opCount = Object.values(keptPaths).reduce(
	(n, item) => n + HTTP_METHODS.filter((m) => m in item).length,
	0,
);
process.stdout.write(
	`kept ${opCount} operations across ${Object.keys(keptPaths).length} paths, ` +
		`${Object.keys(sortedComponents.schemas ?? {}).length} schemas\n`,
);
