# Confluence Command Reference

Complete syntax and flags for all `atla confluence` commands. All commands accept global flags:
`-o/--output`, `--profile`, `--verbose`, `--dry-run`, `--no-input`.

---

## Spaces

### List spaces
```
atla confluence space list [--key KEY] [--limit N=25]
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
atla confluence page list [-s SPACE | --space-id ID] [--title TEXT] [--limit N=25]
```

### View a page
```
atla confluence page view <ID> [--web] [--format markdown|storage|atlas-doc-format]
```
- `--format markdown` returns rendered Markdown
- `--web` opens in browser
- Combine with `--output json` for structured data

### List page children
```
atla confluence page children <ID> [--depth N] [--limit N=25]
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
                              [--draft] [--private]
```
Example:
```bash
atla confluence page create --space ENG --title 'SSO Rollout Checklist' \
  --body-file docs/sso-rollout.md --representation markdown --parent 654321
```

### Update a page
```
atla confluence page update <ID> [--title TITLE] [--parent ID]
                                  [--body TEXT | --body-file FILE]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
                                  [--version N] [--message TEXT] [--draft]
```
Use `page move` for parent-only moves. `page update --parent ...` is for when also updating body/version.

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
atla confluence page label list <PAGE_ID> [--prefix PREFIX] [--limit N=25]
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
atla confluence page comment list <PAGE_ID> [--limit N=25]
```

### Add a comment
```
atla confluence page comment add <PAGE_ID> [BODY | --body-file FILE]
                                  [--parent COMMENT_ID]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
```

### Delete a comment
```
atla confluence page comment delete <PAGE_ID> <COMMENT_ID> [--yes]
```

---

## Blogs

### List blog posts
```
atla confluence blog list [-s SPACE | --space-id ID] [--title TEXT] [--limit N=25]
```

### View a blog post
```
atla confluence blog view <ID>
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
atla confluence blog label list <BLOG_ID> [--prefix PREFIX] [--limit N=25]
atla confluence blog label add <BLOG_ID> LABEL [LABEL ...]
atla confluence blog label remove <BLOG_ID> <LABEL>
```

### Blog comments
```
atla confluence blog comment list <BLOG_ID> [--limit N=25]
atla confluence blog comment add <BLOG_ID> [BODY | --body-file FILE]
                                  [--parent COMMENT_ID]
                                  [--representation storage|wiki|atlas-doc-format|markdown]
```

---

## Search

### Run a CQL search
```
atla confluence search <CQL> [--limit N=25]
```
Example:
```bash
atla confluence search 'type = page AND space = ENG AND title ~ "Runbook"' --limit 25
```

---

## Attachments

### List attachments
```
atla confluence attachment list <PAGE_ID> [--filename NAME] [--limit N=25]
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
atla confluence attachment download <ATTACHMENT_ID> [-o PATH]
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

## API Notes

`atla` primarily uses Confluence v2 API, falling back to scoped v1 REST endpoints where v2 lacks coverage:

- **v1**: `search`, `attachment upload`, `page label add/remove`, `blog label add/remove`, `space update/delete`
- **v2**: Everything else (space/page/blog CRUD, comments, attachment list/view/download/delete, label list)
