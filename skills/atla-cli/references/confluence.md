# Confluence Command Reference

Complete syntax and flags for all `atla confluence` commands. All commands accept global flags:
`-o/--output`, `--profile`, `--verbose`, `--dry-run`, `--no-input`.

**Pagination.** `--limit N` is a hard cap on returned items for the current invocation,
not a single-page hint. If more matches exist, `atla` returns an opaque `--page-token` for
the next logical page. Table output prints a ready-to-copy next command; JSON output
includes `pagination.nextPageToken` and `pagination.nextCommand`; CSV/keys keep stdout
record-only and write the next-page hint to stderr. Pass `--page-token` back to the same
command/query to continue. Use `--all` when you want every matching record; `--all` is
mutually exclusive with both `--limit` and `--page-token`. All paginating syntaxes below
also accept `--all` in place of `--limit`/`--page-token`.

---

## Spaces

### List spaces
```
atla confluence space list [--key KEY] [--limit N=25] [--page-token TOKEN] [--all]
```

### View a space
```
atla confluence space view <KEY>
```

### Create a space
```
atla confluence space create <NAME> [--key KEY] [--alias ALIAS]
                                     [--description TEXT | --description-file FILE] [--private]
```
Requires either `--key` or `--alias`.

### Update a space
```
atla confluence space update <KEY> [--name NAME] [--description TEXT | --description-file FILE]
```

### Delete a space
```
atla confluence space delete <KEY> [--yes]
```

---

## Pages

### List pages
```
atla confluence page list [-s SPACE | --space-id ID] [--title TEXT] [--limit N=25] [--page-token TOKEN] [--all]
```

### View a page
```
atla confluence page view <ID> [--web] [--format markdown|storage|atlas-doc-format] [--preserve-table-options] [--with-attachments]
```
- `--format markdown` returns rendered Markdown
- `--web` opens it in the browser
- `--with-attachments` also fetches and prints page attachments
- `--preserve-table-options` keeps ADF table metadata directives (like numbered rows) in Markdown output (only valid with `--format markdown`).
- Combine with `--output json` for structured data

### List page children
```
atla confluence page children <ID> [--depth N] [--limit N=25] [--page-token TOKEN] [--all]
```

### Copy a page
```
atla confluence page copy <SOURCE_ID> --title TITLE [-s SPACE | --space-id ID]
                           [--parent ID] [--root-level]
```

### Create a page
```
atla confluence page create [-s SPACE | --space-id ID] --title TITLE
                              [--parent ID | --root-level]
                              [--body TEXT | --body-file FILE]
                              [--representation storage|wiki|atlas-doc-format|markdown]
                              [--numbered-table-rows]
                              [--mention NAME=ACCOUNT_ID] [--resolve-mentions]
                              [--draft] [--private]
```

`--numbered-table-rows`, `--mention`, and `--resolve-mentions` apply when using `--representation markdown`.
`--numbered-table-rows` enables numbered rows in Markdown tables.
`--mention` maps a single `NAME=ACCOUNT_ID` pair and `--resolve-mentions` attempts to auto-resolve `@name` mentions.

Example:
```bash
atla confluence page create --space ENG --title 'SSO Rollout Checklist' \
  --body-file docs/sso-rollout.md --representation markdown --parent 654321
atla confluence page create --space ENG --title 'Runbook' --body-file docs/runbook.md --representation markdown --numbered-table-rows
```

### Update a page
```
atla confluence page update <ID> [--title TITLE] [--parent ID]
                                  [--body TEXT | --body-file FILE]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
                                  [--numbered-table-rows]
                                  [--mention NAME=ACCOUNT_ID] [--resolve-mentions]
                                  [--version N] [--message TEXT] [--draft]
```
Use `page move` for parent-only moves. `page update --parent ...` is for when also updating body/version.

`--numbered-table-rows`, `--mention`, and `--resolve-mentions` apply when using `--representation markdown`.

### Delete a page
```
atla confluence page delete <ID> [--purge] [--draft] [--yes]
```
`--purge` permanently removes (bypasses trash).

### Move a page
```
atla confluence page move <ID> --parent NEW_PARENT_ID
```

---

## Page Labels

### List labels
```
atla confluence page label list <PAGE_ID> [--prefix PREFIX] [--limit N=25] [--page-token TOKEN] [--all]
```

### Add labels
```
atla confluence page label add <PAGE_ID> LABEL [LABEL ...]
```
Example: `atla confluence page label add 123456 runbook production urgent`

### Remove a label
```
atla confluence page label remove <PAGE_ID> <LABEL>
```

---

## Page Comments

### List comments
```
atla confluence page comment list <PAGE_ID> [--limit N=25] [--page-token TOKEN] [--all]
```

### Add a comment
```
atla confluence page comment add <PAGE_ID> [BODY | --body TEXT | --body-file FILE]
                                  [--parent COMMENT_ID]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
                                  [--numbered-table-rows]
                                  [--mention NAME=ACCOUNT_ID] [--resolve-mentions]
                                  [--attachment FILE ...]
                                  [--attachment-mode auto|link|embed]
```

### Delete a comment
```
atla confluence page comment delete <PAGE_ID> <COMMENT_ID> [--yes]
```

When `--attachment` is used, `atla` uploads each file to the page first, then appends attachment references to the new comment. `auto` embeds image-style references where supported and links other files; use `link` for links only or `embed` for richer embed-style references where supported.

---

## Blogs

### List blog posts
```
atla confluence blog list [-s SPACE | --space-id ID] [--title TEXT] [--limit N=25] [--page-token TOKEN] [--all]
```

### View a blog post
```
atla confluence blog view <ID> [--format markdown|storage|atlas-doc-format]
```

### Create a blog post
```
atla confluence blog create [-s SPACE | --space-id ID] --title TITLE
                              [--body TEXT | --body-file FILE]
                              [--representation storage|wiki|atlas-doc-format|markdown]
                              [--draft] [--private]
```

### Update a blog post
```
atla confluence blog update <ID> [--title TITLE]
                                  [--body TEXT | --body-file FILE]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
                                  [--version N] [--message TEXT] [--draft]
```

### Delete a blog post
```
atla confluence blog delete <ID> [--purge] [--draft] [--yes]
```

### Blog labels
```
atla confluence blog label list <BLOG_ID> [--prefix PREFIX] [--limit N=25] [--page-token TOKEN] [--all]
atla confluence blog label add <BLOG_ID> LABEL [LABEL ...]
atla confluence blog label remove <BLOG_ID> <LABEL>
```

### Blog comments
```
atla confluence blog comment list <BLOG_ID> [--limit N=25] [--page-token TOKEN] [--all]
atla confluence blog comment add <BLOG_ID> [BODY | --body TEXT | --body-file FILE]
                                  [--parent COMMENT_ID]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
atla confluence blog comment delete <BLOG_ID> <COMMENT_ID> [--yes]
```

---

## Search

### Run a CQL search
```
atla confluence search <CQL> [--limit N=25] [--page-token TOKEN] [--all]
```
Example:
```bash
atla confluence search 'type = page AND space = ENG AND title ~ "Runbook"' --limit 25
```

---

## Attachments

### List attachments
```
atla confluence attachment list <PAGE_ID> [--filename NAME] [--limit N=25] [--page-token TOKEN] [--all]
```

### View attachment metadata
```
atla confluence attachment view <ATTACHMENT_ID>
```

### Upload an attachment
```
atla confluence attachment upload <PAGE_ID> <FILE> [--comment TEXT] [--minor-edit]
```

### Download an attachment
```
atla confluence attachment download <ATTACHMENT_ID> [--save-to FILE | -f FILE]
```

### Delete an attachment
```
atla confluence attachment delete <ATTACHMENT_ID> [--purge] [--yes]
```

---

## Content Body Representations

### For write commands (`create`, `update`)

| Representation | Meaning | Best for |
|----------------|---------|----------|
| `storage` | Confluence Storage Format XML | Precise native content (default) |
| `wiki` | Legacy wiki markup | Existing wiki automation |
| `atlas-doc-format` | Atlassian Document Format JSON | Structured editor-native work |
| `markdown` | Markdown input | Authoring from plain text files |

### For view commands

| Format | Result |
|--------|--------|
| `markdown` | Markdown-rendered body |
| `storage` | Raw storage format |
| `atlas-doc-format` | Raw ADF JSON |

---

## CQL Quick Reference

| Goal | CQL |
|------|-----|
| Pages in a space | `type = page AND space = ENG` |
| Blog posts by title | `type = blogpost AND title ~ "release"` |
| Recently updated | `type = page AND lastmodified >= now("-7d")` |
| By creator | `creator = currentUser()` |
| By label | `label = runbook` |
| Draft content | `status = draft` |
| Combined filter | `type = page AND space = ENG AND title ~ "SSO"` |

---

## File Embedding: Always Use Attachments

### Never reference files by external URL

Confluence Cloud enforces a Content Security Policy (CSP) that prevents externally hosted files from rendering — images, PDFs, and any other embedded content included. `<ri:url>` references silently fail for all users and cannot be fixed without admin-level domain allowlisting. This is a platform constraint, not a permissions issue.

**Always upload files as page attachments and reference them by filename.**

### Upload → reference workflow

```bash
# Download the file
curl -sL -o /tmp/file.jpg "https://..."

# Upload to the target page
atla confluence attachment upload PAGE_ID /tmp/file.jpg
```

Then reference in Storage Format:

| Content type | Storage Format |
|---|---|
| Image | `<ac:image ac:align="center" ac:width="600"><ri:attachment ri:filename="file.jpg"/></ac:image>` |
| File preview | `<ac:structured-macro ac:name="view-file"><ac:parameter ac:name="name"><ri:attachment ri:filename="file.pdf"/></ac:parameter></ac:structured-macro>` |
| Download link | `<ac:link><ri:attachment ri:filename="file.zip"/><ac:plain-text-link-body><![CDATA[Download]]></ac:plain-text-link-body></ac:link>` |

To reference an attachment on a *different* page, nest `ri:page` inside `ri:attachment`:
```xml
<ac:image>
  <ri:attachment ri:filename="file.jpg">
    <ri:page ri:content-title="Source Page Title" ri:space-key="SPACE"/>
  </ri:attachment>
</ac:image>
```

---

## API Notes

`atla` primarily uses Confluence v2 API, falling back to scoped v1 REST endpoints where v2 lacks coverage:

- **v1**: `search`, `attachment upload`, `page label add/remove`, `blog label add/remove`, `space update/delete`
- **v2**: Everything else (space/page/blog CRUD, comments, attachment list/view/download/delete, label list)
