---
title: Confluence Reference
description: Complete Confluence command reference for atla.
---

# Confluence

`atla confluence` covers spaces, pages, blogs, CQL search, comments, labels, and attachments.
All commands also accept the global flags described in [`output-formats.md`](./output-formats.md):
`-o, --output`, `--profile`, `--verbose`, `--dry-run`, and `--no-input`.

## Pagination

Every `--limit N` flag is a "max-results" cap, not a single-page hint. `atla` paginates
the Confluence v2 cursor (or v1 CQL `start`/`totalSize`) internally and accumulates up to
`N` items before returning, so `--limit 1000` reliably returns 1000 results rather than
the ~25/250 a single Confluence page would yield.

If the server still has more matches when the limit is hit, `atla` returns a `--page-token`
for the next logical page instead of forcing you to increase `--limit`. In table output,
the token is shown as a ready-to-copy command:

```text
More results available.
Next page:
  atla confluence page list --space-id 123 --limit 25 --page-token <TOKEN>
```

In JSON output, pagination metadata is included alongside the records:

```json
{
  "results": [],
  "pagination": {
    "isLast": false,
    "nextPageToken": "...",
    "nextCommand": "atla confluence page list --space-id 123 --limit 25 --page-token ..."
  }
}
```

For `csv` and `keys` output, records stay on stdout and the next-page hint is written to
stderr so pipelines remain clean. `--page-token` is intentionally opaque; pass it back to
the same command/query to continue. Tokens are validated against the command and query,
and using one with a different query fails fast.

### `--all`

When you want every matching record without guessing an upper bound, use `--all`. It
follows the cursor (or `start`/`totalSize`) until the server reports no more results,
ignores the `--limit` clamp, and does not emit next-page metadata because it fetches until
exhaustion:

```bash
atla confluence search 'type = page AND space = ENG' --all --output keys > all-pages.txt
atla confluence space list --all --output json | jq '.results | length'
```

`--all` is mutually exclusive with both `--limit` and `--page-token`. Be aware that
`--all` against a large result set issues many HTTP requests (one per 100 items), so use
it deliberately on broad queries.

### Affected commands

`confluence space list`, `confluence page list`, `confluence page children`,
`confluence blog list`, `confluence page comment list`, `confluence blog comment list`,
`confluence page label list`, `confluence blog label list`, `confluence attachment list`,
and `confluence search`.

## Spaces

### List spaces

**Syntax**

```bash
atla confluence space list [--key KEY] [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence space list --key ENG --limit 10
```

### View a space

**Syntax**

```bash
atla confluence space view <KEY>
```

**Example**

```bash
atla confluence space view ENG
```

### Create a space

**Syntax**

```bash
atla confluence space create <NAME> [--key KEY] [--alias ALIAS]
                                     [--description TEXT | --description-file FILE] [--private]
```

**Example**

```bash
atla confluence space create 'Engineering Runbooks' --key ENGOPS   --description 'Operational playbooks for the engineering team'
```

`atla` currently requires either `--key` or `--alias` when creating a space.

### Update a space

**Syntax**

```bash
atla confluence space update <KEY> [--name NAME]
                                     [--description TEXT | --description-file FILE]
```

**Example**

```bash
atla confluence space update ENGOPS --name 'Engineering Operations'   --description-file docs/engops-space-description.txt
```

### Delete a space

**Syntax**

```bash
atla confluence space delete <KEY> [--yes]
```

**Example**

```bash
atla confluence space delete ENGOPS --yes
```

## Pages

### List pages

**Syntax**

```bash
atla confluence page list [-s SPACE | --space-id ID] [--title TEXT] [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence page list --space ENG --title 'Runbook' --limit 20
```

### View a page

**Syntax**

```bash
atla confluence page view <ID> [--web] [--format markdown|storage|atlas-doc-format]
                           [--preserve-table-options]
```

**Examples**

```bash
atla confluence page view 123456
atla confluence page view 123456 --format markdown
atla confluence page view 123456 --format markdown --preserve-table-options
atla confluence page view 123456 --web
```

Use `--preserve-table-options` with `--format markdown` to emit `<!-- atla:table ... -->` directives for ADF table metadata such as numbered rows.

### List page children

**Syntax**

```bash
atla confluence page children <ID> [--depth N] [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence page children 123456 --depth 2 --limit 50
```

### Copy a page

**Syntax**

```bash
atla confluence page copy <SOURCE_ID> --title TITLE [-s SPACE | --space-id ID]
                           [--parent ID] [--root-level]
```

**Example**

```bash
atla confluence page copy 123456 --title 'Incident Runbook Template'   --space ENG --parent 654321
```

### Create a page

**Syntax**

```bash
atla confluence page create [-s SPACE | --space-id ID] --title TITLE
                              [--parent ID | --root-level]
                              [--body TEXT | --body-file FILE]
                              [--representation storage|wiki|atlas-doc-format|markdown]
                              [--numbered-table-rows]
                              [--mention NAME=ACCOUNT_ID] [--resolve-mentions]
                              [--draft] [--private]
```

**Example**

```bash
atla confluence page create --space ENG --title 'SSO Rollout Checklist'   --body-file docs/sso-rollout.md   --representation markdown   --parent 654321
atla confluence page create --space ENG --title 'Inventory'   --body-file docs/inventory.md   --representation markdown --numbered-table-rows
atla confluence page create --space ENG --title 'Runbook'   --body-file docs/runbook.md   --representation markdown --mention 'Neo Hsu=abc-account-id'
```

### Update a page

**Syntax**

```bash
atla confluence page update <ID> [--title TITLE] [--parent ID]
                                  [--body TEXT | --body-file FILE]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
                                  [--numbered-table-rows]
                                  [--mention NAME=ACCOUNT_ID] [--resolve-mentions]
                                  [--version N] [--message TEXT] [--draft]
```

**Examples**

```bash
atla confluence page update 123456 --title 'SSO Rollout Checklist v2'
atla confluence page update 123456 --body-file docs/sso-rollout.md   --representation markdown --message 'Refresh rollout steps'
atla confluence page update 123456 --body-file docs/inventory.md   --representation markdown --numbered-table-rows
```

Use `page move` for parent-only moves. `page update --parent ...` is best when you are also updating the body/version.

`--numbered-table-rows`, `--mention`, and `--resolve-mentions` only apply when `--representation markdown` converts Markdown to Atlas Doc Format.

Markdown mentions remain literal text unless you explicitly map or resolve them. Use `--mention 'Name=ACCOUNT_ID'` for deterministic automation, or `--resolve-mentions` to scan `@name` / `@[Display Name]` candidates and resolve unique Atlassian user search matches. Ambiguous or missing matches are left as text with a warning.

To enable numbered rows for just one table in a Markdown file, place a directive immediately before that table:

```markdown
<!-- atla:table numbered-rows=true -->
| Name | Status |
| --- | --- |
| API | Done |
```

### Delete a page

**Syntax**

```bash
atla confluence page delete <ID> [--purge] [--draft] [--yes]
```

**Example**

```bash
atla confluence page delete 123456 --purge --yes
```

### Move a page

**Syntax**

```bash
atla confluence page move <ID> --parent NEW_PARENT_ID
```

**Example**

```bash
atla confluence page move 123456 --parent 654321
```

### Page labels

#### List labels

**Syntax**

```bash
atla confluence page label list <PAGE_ID> [--prefix PREFIX] [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence page label list 123456 --prefix global --limit 20
```

#### Add labels

**Syntax**

```bash
atla confluence page label add <PAGE_ID> LABEL [LABEL ...]
```

**Example**

```bash
atla confluence page label add 123456 runbook production urgent
```

#### Remove a label

**Syntax**

```bash
atla confluence page label remove <PAGE_ID> <LABEL>
```

**Example**

```bash
atla confluence page label remove 123456 urgent
```

### Page comments

#### List comments

**Syntax**

```bash
atla confluence page comment list <PAGE_ID> [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence page comment list 123456 --limit 10
```

#### Add a comment

**Syntax**

```bash
atla confluence page comment add <PAGE_ID> [BODY | --body-file FILE]
                                  [--parent COMMENT_ID]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
                                  [--numbered-table-rows]
                                  [--mention NAME=ACCOUNT_ID] [--resolve-mentions]
                                  [--attachment FILE ...]
                                  [--attachment-mode auto|link|embed]
```

**Example**

```bash
atla confluence page comment add 123456 'Please verify the rollback steps.'
atla confluence page comment add 123456 'Please review the report.'   --representation markdown --attachment ./report.pdf
```

When `--attachment` is used, `atla` uploads files to the page before creating the comment, then appends attachment references to the comment body. `auto` embeds image-style references where supported and links other files; use `link` for links only or `embed` to request richer embed-style references where the selected representation supports them.

#### Delete a comment

**Syntax**

```bash
atla confluence page comment delete <PAGE_ID> <COMMENT_ID> [--yes]
```

**Example**

```bash
atla confluence page comment delete 123456 78910 --yes
```

## Blogs

### List blog posts

**Syntax**

```bash
atla confluence blog list [-s SPACE | --space-id ID] [--title TEXT] [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence blog list --space ENG --title 'Release' --limit 10
```

### View a blog post

**Syntax**

```bash
atla confluence blog view <ID>
```

**Example**

```bash
atla confluence blog view 234567
```

### Create a blog post

**Syntax**

```bash
atla confluence blog create [-s SPACE | --space-id ID] --title TITLE
                              [--body TEXT | --body-file FILE]
                              [--representation storage|wiki|atlas-doc-format|markdown]
                              [--draft] [--private]
```

**Example**

```bash
atla confluence blog create --space ENG --title 'Release 2.4 Notes'   --body-file docs/release-2.4.md --representation markdown
```

### Update a blog post

**Syntax**

```bash
atla confluence blog update <ID> [--title TITLE]
                                  [--body TEXT | --body-file FILE]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
                                  [--version N] [--message TEXT] [--draft]
```

**Example**

```bash
atla confluence blog update 234567 --title 'Release 2.4 Notes (Updated)'   --message 'Add migration notes'
```

### Delete a blog post

**Syntax**

```bash
atla confluence blog delete <ID> [--purge] [--draft] [--yes]
```

**Example**

```bash
atla confluence blog delete 234567 --yes
```

### Blog labels

#### List labels

**Syntax**

```bash
atla confluence blog label list <BLOG_ID> [--prefix PREFIX] [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence blog label list 234567 --limit 20
```

#### Add labels

**Syntax**

```bash
atla confluence blog label add <BLOG_ID> LABEL [LABEL ...]
```

**Example**

```bash
atla confluence blog label add 234567 release-notes engineering
```

#### Remove a label

**Syntax**

```bash
atla confluence blog label remove <BLOG_ID> <LABEL>
```

**Example**

```bash
atla confluence blog label remove 234567 engineering
```

### Blog comments

#### List comments

**Syntax**

```bash
atla confluence blog comment list <BLOG_ID> [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence blog comment list 234567 --limit 10
```

#### Add a comment

**Syntax**

```bash
atla confluence blog comment add <BLOG_ID> [BODY | --body-file FILE]
                                  [--parent COMMENT_ID]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
```

**Example**

```bash
atla confluence blog comment add 234567 'Publish after the maintenance window closes.'
```

#### Delete a comment

**Syntax**

```bash
atla confluence blog comment delete <BLOG_ID> <COMMENT_ID> [--yes]
```

**Example**

```bash
atla confluence blog comment delete 234567 78910 --yes
```

## Search

### Run a CQL search

**Syntax**

```bash
atla confluence search <CQL> [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence search 'type = page AND space = ENG AND title ~ "Runbook"' --limit 25
```

## Attachments

### List attachments

**Syntax**

```bash
atla confluence attachment list <PAGE_ID> [--filename NAME] [--limit N=25] [--page-token TOKEN]
```

**Example**

```bash
atla confluence attachment list 123456 --filename diagram --limit 20
```

### View an attachment

**Syntax**

```bash
atla confluence attachment view <ATTACHMENT_ID>
```

**Example**

```bash
atla confluence attachment view 987654
```

### Upload an attachment

**Syntax**

```bash
atla confluence attachment upload <PAGE_ID> <FILE> [--comment TEXT] [--minor-edit]
```

**Example**

```bash
atla confluence attachment upload 123456 ./diagrams/sso-flow.png   --comment 'Updated flow for the new callback URL' --minor-edit
```

### Download an attachment

**Syntax**

```bash
atla confluence attachment download <ATTACHMENT_ID> [--save-to FILE]
```

**Example**

```bash
atla confluence attachment download 987654 --save-to ./downloads/sso-flow.png
```

### Delete an attachment

**Syntax**

```bash
atla confluence attachment delete <ATTACHMENT_ID> [--purge] [--yes]
```

**Example**

```bash
atla confluence attachment delete 987654 --purge --yes
```

## Content body representations

Write commands accept these representations:

| Representation | Meaning | Best for |
| --- | --- | --- |
| `storage` | Confluence Storage Format XML | Precise Confluence-native content; default for writes |
| `wiki` | Legacy Confluence wiki markup | Existing wiki-style automation |
| `atlas-doc-format` | Atlassian Document Format JSON | Structured editor-native automation |
| `markdown` | Markdown input | Authoring content in plain text files |

View commands support these body/output formats:

| View format | Result |
| --- | --- |
| `markdown` | Markdown-rendered body |
| `storage` | Raw storage format |
| `atlas-doc-format` | Raw ADF JSON |

Examples:

```bash
atla confluence page create --space ENG --title 'How to rotate tokens'   --body-file docs/token-rotation.md --representation markdown

atla confluence page view 123456 --format atlas-doc-format --output json
```

## CQL quick reference

CQL is available through `atla confluence search`.

| Goal | Example |
| --- | --- |
| Pages in a space | `type = page AND space = ENG` |
| Blog posts by title | `type = blogpost AND title ~ "release"` |
| Recently updated pages | `type = page AND lastmodified >= now("-7d")` |
| Content by creator | `creator = currentUser()` |
| Content with a label | `label = runbook` |
| Draft content | `status = draft` |
| Pages under a space and title match | `type = page AND space = ENG AND title ~ "SSO"` |

Examples:

```bash
atla confluence search 'type = page AND label = runbook AND space = ENG'
atla confluence search 'type = blogpost AND creator = currentUser()' --limit 50
```

## API notes: v2 first, v1 where needed

Confluence v2 is the primary API client used by `atla`.
Current command coverage is split like this:

### Primarily v2

- Space list/view/create
- Page list/view/children/copy/create/update/delete/move
- Blog list/view/create/update/delete
- Page and blog comment list/add/delete
- Attachment list/view/download/delete
- Page and blog label list

### Scoped v1 REST endpoints

- `atla confluence search` uses the scoped v1 search API.
- `atla confluence attachment upload` uses the scoped v1 attachment endpoint.
- `atla confluence page label add/remove` uses v1 because v2 does not expose those mutations.
- `atla confluence blog label add/remove` follows the same v1 label mutation path.
- Space update/delete currently use Confluence REST paths backed by the v1-style endpoint.

That split is normal for Atlassian Cloud APIs today: `atla` prefers v2, then falls back to targeted v1 calls where required capability is missing.

## `--dry-run` tips

Use `--dry-run` to preview requests before writing content:

```bash
atla --dry-run confluence page create --space ENG --title 'SSO rollout' --body-file docs/sso-rollout.md --representation markdown
atla --dry-run confluence attachment upload 123456 ./diagram.png
atla --dry-run confluence space delete ENGOPS --yes
```

`--dry-run` prints the API call that would run and skips the actual mutation.
