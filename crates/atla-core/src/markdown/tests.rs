use super::render::{content_items, node_type};
use super::*;

fn targeted_schema_errors(value: &Value) -> Vec<String> {
    fn walk(node: &Value, path: &str, errors: &mut Vec<String>) {
        let Some(object) = node.as_object() else {
            if let Some(items) = node.as_array() {
                for (index, child) in items.iter().enumerate() {
                    walk(child, &format!("{path}[{index}]"), errors);
                }
            }
            return;
        };

        match object
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "taskList"
                if object
                    .get("attrs")
                    .and_then(|attrs| attrs.get("localId"))
                    .and_then(Value::as_str)
                    .is_none() =>
            {
                errors.push(format!("{path}: taskList missing attrs.localId"));
            }
            "taskItem" => {
                let attrs = object.get("attrs");
                if attrs
                    .and_then(|attrs| attrs.get("localId"))
                    .and_then(Value::as_str)
                    .is_none()
                {
                    errors.push(format!("{path}: taskItem missing attrs.localId"));
                }
                if !matches!(
                    attrs
                        .and_then(|attrs| attrs.get("state"))
                        .and_then(Value::as_str),
                    Some("TODO" | "DONE")
                ) {
                    errors.push(format!("{path}: taskItem missing valid attrs.state"));
                }
            }
            "blockquote" => {
                for (index, child) in content_items(object).iter().enumerate() {
                    let child_type = child
                        .as_object()
                        .and_then(|value| value.get("type"))
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    if !matches!(
                        child_type,
                        "paragraph"
                            | "orderedList"
                            | "bulletList"
                            | "codeBlock"
                            | "mediaSingle"
                            | "mediaGroup"
                            | "extension"
                    ) {
                        errors.push(format!(
                            "{path}.content[{index}]: blockquote child `{child_type}` invalid"
                        ));
                    }
                }
            }
            "codeBlock" => {
                for (index, child) in content_items(object).iter().enumerate() {
                    let Some(child_object) = child.as_object() else {
                        errors.push(format!(
                            "{path}.content[{index}]: codeBlock child must be text"
                        ));
                        continue;
                    };
                    if node_type(child_object) != "text" {
                        errors.push(format!(
                            "{path}.content[{index}]: codeBlock child must be text"
                        ));
                    }
                    if child_object
                        .get("marks")
                        .and_then(Value::as_array)
                        .is_some_and(|marks| !marks.is_empty())
                    {
                        errors.push(format!(
                            "{path}.content[{index}]: codeBlock text must not have marks"
                        ));
                    }
                    if child_object
                        .get("text")
                        .and_then(Value::as_str)
                        .is_none_or(|text| text.is_empty())
                    {
                        errors.push(format!(
                            "{path}.content[{index}]: text node must be non-empty"
                        ));
                    }
                }
            }
            "text"
                if object
                    .get("text")
                    .and_then(Value::as_str)
                    .is_none_or(|text| text.is_empty()) =>
            {
                errors.push(format!("{path}: text node must be non-empty"));
            }
            "text" => {
                if let Some(Value::Array(marks)) = object.get("marks") {
                    let has_code = marks
                        .iter()
                        .any(|m| m.get("type").and_then(Value::as_str) == Some("code"));
                    let has_other = marks
                        .iter()
                        .any(|m| m.get("type").and_then(Value::as_str) != Some("code"));
                    if has_code && has_other {
                        errors.push(format!(
                            "{path}: text node has `code` mark combined with other marks (ADF violation)"
                        ));
                    }
                }
            }
            "inlineCard" => {
                if object
                    .get("attrs")
                    .and_then(|attrs| attrs.get("url"))
                    .and_then(Value::as_str)
                    .is_none()
                {
                    errors.push(format!("{path}: inlineCard missing attrs.url"));
                }
                if object.contains_key("content") {
                    errors.push(format!("{path}: inlineCard must not have content"));
                }
            }
            "orderedList"
                if object
                    .get("attrs")
                    .and_then(|attrs| attrs.get("order"))
                    .and_then(Value::as_f64)
                    .is_some_and(|order| order < 0.0) =>
            {
                errors.push(format!("{path}: orderedList order must be >= 0"));
            }
            _ => {}
        }

        for (key, child) in object {
            walk(child, &format!("{path}.{key}"), errors);
        }
    }

    let mut errors = Vec::new();
    walk(value, "$", &mut errors);
    errors
}

#[test]
fn converts_adf_to_markdown_text() {
    let adf = json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": { "level": 2 },
                "content": [{ "type": "text", "text": "Runbook" }]
            },
            {
                "type": "paragraph",
                "content": [{ "type": "text", "text": "Deploy steps" }]
            }
        ]
    });

    assert_eq!(adf_to_markdown(&adf), "## Runbook\n\nDeploy steps");
}

#[test]
fn converts_markdown_blocks_to_adf() {
    let adf = markdown_to_adf(
        r#"# Runbook

Deploy **fast** with [docs](https://example.com).

- Build
- Test

1. Ship

- [x] Verify

> Heads up

```bash
cargo test
```

---
"#,
    );

    let content = adf["content"].as_array().expect("content array");
    assert_eq!(content[0]["type"], json!("heading"));
    assert_eq!(content[0]["attrs"]["level"], json!(1));
    assert_eq!(content[1]["type"], json!("paragraph"));
    assert_eq!(
        content[1]["content"][1]["marks"][0]["type"],
        json!("strong")
    );
    assert_eq!(content[1]["content"][3]["marks"][0]["type"], json!("link"));
    assert_eq!(content[2]["type"], json!("bulletList"));
    assert_eq!(content[3]["type"], json!("orderedList"));
    assert_eq!(content[4]["type"], json!("taskList"));
    assert_eq!(content[4]["attrs"]["localId"], json!("tasklist-10"));
    assert_eq!(content[4]["content"][0]["attrs"]["state"], json!("DONE"));
    assert_eq!(content[5]["type"], json!("blockquote"));
    assert_eq!(content[6]["type"], json!("codeBlock"));
    assert_eq!(content[6]["attrs"]["language"], json!("bash"));
    assert_eq!(content[7]["type"], json!("rule"));
    assert!(targeted_schema_errors(&adf).is_empty());
}

#[test]
fn round_trips_common_markdown_subset() {
    let markdown = "## Runbook\n\nDeploy **fast** and `safe`.\n\n- Build\n- Test";
    let adf = markdown_to_adf(markdown);

    assert_eq!(
        adf_to_markdown(&adf),
        "## Runbook\n\nDeploy **fast** and `safe`.\n\n- Build\n- Test"
    );
}

#[test]
fn converts_marks_links_and_mentions() {
    let adf = json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "important",
                        "marks": [{ "type": "strong" }, { "type": "em" }]
                    },
                    { "type": "text", "text": " " },
                    {
                        "type": "text",
                        "text": "docs",
                        "marks": [
                            {
                                "type": "link",
                                "attrs": { "href": "https://example.com/docs" }
                            }
                        ]
                    },
                    { "type": "text", "text": " " },
                    {
                        "type": "mention",
                        "attrs": { "text": "Neo" }
                    }
                ]
            }
        ]
    });

    assert_eq!(
        adf_to_markdown(&adf),
        "_**important**_ [docs](https://example.com/docs) @Neo"
    );
}

#[test]
fn markdown_mentions_stay_text_without_mapping() {
    let adf = markdown_to_adf("@Neo please check @[Amy Chen]");
    let content = adf["content"][0]["content"]
        .as_array()
        .expect("paragraph content array");

    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], json!("text"));
    assert_eq!(content[0]["text"], json!("@Neo please check @[Amy Chen]"));
}

#[test]
fn markdown_mentions_convert_with_mapping() {
    let adf = markdown_to_adf_with_options(
        "@Neo please check @[Amy Chen] and `@Robot`",
        MarkdownToAdfOptions {
            mentions: vec![
                MarkdownMention {
                    text: "Neo".to_owned(),
                    account_id: "account-neo".to_owned(),
                },
                MarkdownMention {
                    text: "Amy Chen".to_owned(),
                    account_id: "account-amy".to_owned(),
                },
                MarkdownMention {
                    text: "Robot".to_owned(),
                    account_id: "account-robot".to_owned(),
                },
            ],
            ..MarkdownToAdfOptions::default()
        },
    );
    let content = adf["content"][0]["content"]
        .as_array()
        .expect("paragraph content array");

    assert_eq!(content[0]["type"], json!("mention"));
    assert_eq!(content[0]["attrs"]["id"], json!("account-neo"));
    assert_eq!(content[0]["attrs"]["text"], json!("@Neo"));
    assert_eq!(content[2]["type"], json!("mention"));
    assert_eq!(content[2]["attrs"]["id"], json!("account-amy"));
    assert_eq!(content[4]["type"], json!("text"));
    assert_eq!(content[4]["text"], json!("@Robot"));
    assert_eq!(content[4]["marks"], json!([{ "type": "code" }]));
    assert!(targeted_schema_errors(&adf).is_empty());
}

#[test]
fn markdown_mentions_do_not_convert_email_addresses() {
    let adf = markdown_to_adf_with_options(
        "email neo@example.com then @example",
        MarkdownToAdfOptions {
            mentions: vec![MarkdownMention {
                text: "example".to_owned(),
                account_id: "account-example".to_owned(),
            }],
            ..MarkdownToAdfOptions::default()
        },
    );
    let content = adf["content"][0]["content"]
        .as_array()
        .expect("paragraph content array");

    assert_eq!(content[0]["type"], json!("text"));
    assert_eq!(content[1]["type"], json!("text"));
    assert_eq!(content[1]["text"], json!("neo@example.com"));
    assert_eq!(content[1]["marks"][0]["type"], json!("link"));
    assert_eq!(
        content[1]["marks"][0]["attrs"]["href"],
        json!("mailto:neo@example.com")
    );
    assert_eq!(content[2]["text"], json!(" then "));
    assert_eq!(content[3]["type"], json!("mention"));
}

#[test]
fn scans_markdown_mention_candidates() {
    let candidates = markdown_mention_candidates(
        "@Neo and @[Amy Chen] plus you@example.com and `@Robot`\n```\n@Code\n```\n@Neo.",
    );

    assert_eq!(candidates, vec!["Neo", "Amy Chen"]);
}

#[test]
fn converts_nested_lists_tasks_and_code_blocks() {
    let adf = json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "bulletList",
                "content": [
                    {
                        "type": "listItem",
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{ "type": "text", "text": "Deploy" }]
                            },
                            {
                                "type": "orderedList",
                                "content": [
                                    {
                                        "type": "listItem",
                                        "content": [
                                            {
                                                "type": "paragraph",
                                                "content": [{ "type": "text", "text": "Build" }]
                                            }
                                        ]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            },
            {
                "type": "taskList",
                "content": [
                    {
                        "type": "taskItem",
                        "attrs": { "state": "DONE" },
                        "content": [{ "type": "text", "text": "Verify" }]
                    }
                ]
            },
            {
                "type": "codeBlock",
                "attrs": { "language": "bash" },
                "content": [{ "type": "text", "text": "cargo test" }]
            }
        ]
    });

    assert_eq!(
        adf_to_markdown(&adf),
        "- Deploy\n  1. Build\n\n- [x] Verify\n```bash\ncargo test\n```"
    );
}

#[test]
fn converts_tables_blockquotes_and_cards() {
    let adf = json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "blockquote",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [{ "type": "text", "text": "Heads up" }]
                    }
                ]
            },
            {
                "type": "table",
                "content": [
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableHeader",
                                "content": [
                                    {
                                        "type": "paragraph",
                                        "content": [{ "type": "text", "text": "Key" }]
                                    }
                                ]
                            },
                            {
                                "type": "tableHeader",
                                "content": [
                                    {
                                        "type": "paragraph",
                                        "content": [{ "type": "text", "text": "Value" }]
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableCell",
                                "content": [
                                    {
                                        "type": "paragraph",
                                        "content": [{ "type": "text", "text": "Status" }]
                                    }
                                ]
                            },
                            {
                                "type": "tableCell",
                                "content": [
                                    {
                                        "type": "paragraph",
                                        "content": [{ "type": "text", "text": "Done" }]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            },
            {
                "type": "inlineCard",
                "attrs": { "url": "https://example.com/card" }
            }
        ]
    });

    assert_eq!(
        adf_to_markdown(&adf),
        "> Heads up\n\n| Key | Value |\n| --- | --- |\n| Status | Done |\n\nhttps://example.com/card"
    );
}

#[test]
fn adf_table_numbered_rows_render_as_directive_when_requested() {
    let adf = json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "table",
                "attrs": {"isNumberColumnEnabled": true, "layout": "default"},
                "content": [
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableHeader",
                                "content": [{
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "Key"}]
                                }]
                            }
                        ]
                    },
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableCell",
                                "content": [{
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "Status"}]
                                }]
                            }
                        ]
                    }
                ]
            }
        ]
    });

    assert_eq!(adf_to_markdown(&adf), "| Key |\n| --- |\n| Status |");

    let markdown = adf_to_markdown_with_options(
        &adf,
        AdfToMarkdownOptions {
            table_numbered_rows_directives: true,
        },
    );
    assert_eq!(
        markdown,
        "<!-- atla:table numbered-rows=true -->\n| Key |\n| --- |\n| Status |"
    );

    let round_tripped = markdown_to_adf(&markdown);
    assert_eq!(
        round_tripped["content"][0]["attrs"]["isNumberColumnEnabled"],
        json!(true)
    );
}

#[test]
fn markdown_table_numbered_rows_are_opt_in() {
    let markdown = "| Key | Value |\n| --- | --- |\n| Status | Done |";

    let default_adf = markdown_to_adf(markdown);
    assert_eq!(
        default_adf["content"][0]["attrs"]["isNumberColumnEnabled"],
        json!(false)
    );

    let numbered_adf = markdown_to_adf_with_options(
        markdown,
        MarkdownToAdfOptions {
            numbered_table_rows: true,
            ..MarkdownToAdfOptions::default()
        },
    );
    assert_eq!(
        numbered_adf["content"][0]["attrs"]["isNumberColumnEnabled"],
        json!(true)
    );
}

#[test]
fn markdown_table_directive_enables_numbered_rows_for_next_table() {
    let adf = markdown_to_adf(
        "<!-- atla:table numbered-rows=true -->\n| A |\n| --- |\n| one |\n\n| B |\n| --- |\n| two |",
    );

    assert_eq!(
        adf["content"][0]["attrs"]["isNumberColumnEnabled"],
        json!(true)
    );
    assert_eq!(
        adf["content"][1]["attrs"]["isNumberColumnEnabled"],
        json!(false)
    );
}

#[test]
fn parses_commonmark_and_gfm_markdown_with_comrak() {
    let adf = markdown_to_adf(
        "Setext\n======\n\nSee [Rust][rust] and www.example.com.\n\n| A | B |\n| - | - |\n| escaped \\| pipe | ~~old~~ |\n\nFootnote[^n].\n\n[^n]: note body\n\n[rust]: https://www.rust-lang.org",
    );

    assert_eq!(adf["content"][0]["type"], json!("heading"));
    assert_eq!(adf["content"][0]["attrs"]["level"], json!(1));
    assert_eq!(
        adf["content"][1]["content"][1]["marks"][0]["attrs"]["href"],
        json!("https://www.rust-lang.org")
    );
    assert_eq!(
        adf["content"][1]["content"][3]["marks"][0]["attrs"]["href"],
        json!("http://www.example.com")
    );
    assert_eq!(
        adf["content"][2]["content"][1]["content"][0]["content"][0]["content"][0]["text"],
        json!("escaped | pipe")
    );
    assert_eq!(
        adf["content"][2]["content"][1]["content"][1]["content"][0]["content"][0]["marks"][0]["type"],
        json!("strike")
    );
    assert_eq!(adf["content"][3]["content"][1]["text"], json!("[^n]"));
    assert_eq!(
        adf["content"][4]["content"][0]["text"],
        json!("[^n]: note body")
    );
    assert!(targeted_schema_errors(&adf).is_empty());
}

#[test]
fn converts_underscore_and_nested_marks() {
    let adf = markdown_to_adf("_italic_ __bold__ **_both_**");

    assert_eq!(
        adf["content"][0]["content"][0]["marks"][0]["type"],
        json!("em")
    );
    assert_eq!(
        adf["content"][0]["content"][2]["marks"][0]["type"],
        json!("strong")
    );
    assert_eq!(
        adf["content"][0]["content"][4]["marks"],
        json!([{ "type": "em" }, { "type": "strong" }])
    );
}

#[test]
fn blockquote_headings_become_paragraphs() {
    let adf = markdown_to_adf("> # Quoted heading");

    assert_eq!(adf["content"][0]["type"], json!("blockquote"));
    assert_eq!(adf["content"][0]["content"][0]["type"], json!("paragraph"));
    assert!(targeted_schema_errors(&adf).is_empty());
}

#[test]
fn multiline_blockquote_preserves_both_lines() {
    // Each `> ` line must survive the MD→ADF→MD roundtrip as a visible separate line.
    let markdown = "> First line.\n> Second line.";
    let adf = markdown_to_adf(markdown);

    // CommonMark parses adjacent quoted lines as one paragraph with a soft line break.
    assert_eq!(adf["content"][0]["type"], json!("blockquote"));
    let quote_content = adf["content"][0]["content"]
        .as_array()
        .expect("blockquote content array");
    assert_eq!(quote_content.len(), 1);
    assert_eq!(quote_content[0]["content"][1]["type"], json!("hardBreak"));

    let rendered = adf_to_markdown(&adf);
    assert!(
        rendered.contains("> First line."),
        "First line missing: {rendered}"
    );
    assert!(
        rendered.contains("> Second line."),
        "Second line missing: {rendered}"
    );
}

#[test]
fn empty_code_blocks_omit_empty_text_nodes() {
    let adf = markdown_to_adf("```\n```");

    assert_eq!(adf["content"][0], json!({ "type": "codeBlock" }));
    assert!(targeted_schema_errors(&adf).is_empty());
}

#[test]
fn renders_multiline_code_blocks_panels_and_mentions() {
    let adf = json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "codeBlock",
                "attrs": { "language": "rust" },
                "content": [{ "type": "text", "text": "fn main() {\n    println!(\"hi\");\n}" }]
            },
            {
                "type": "panel",
                "attrs": { "panelType": "info" },
                "content": [
                    {
                        "type": "paragraph",
                        "content": [{ "type": "text", "text": "Heads up" }]
                    }
                ]
            },
            {
                "type": "paragraph",
                "content": [{ "type": "mention", "attrs": { "text": "Neo" } }]
            }
        ]
    });

    assert_eq!(
        adf_to_markdown(&adf),
        "```rust\nfn main() {\n    println!(\"hi\");\n}\n```\n\n> **info:**\n>\n> Heads up\n\n@Neo"
    );
}

#[test]
fn stateful_marks_bold_wraps_italic_span() {
    // bold outer, italic inner for one word only: **bold _italic_ more bold**
    let adf = json!({
        "type": "doc", "version": 1,
        "content": [{
            "type": "paragraph",
            "content": [
                { "type": "text", "text": "bold ", "marks": [{ "type": "strong" }] },
                { "type": "text", "text": "italic", "marks": [{ "type": "strong" }, { "type": "em" }] },
                { "type": "text", "text": " more bold", "marks": [{ "type": "strong" }] }
            ]
        }]
    });
    assert_eq!(adf_to_markdown(&adf), "**bold _italic_ more bold**");
}

#[test]
fn stateful_marks_bold_wraps_code_span() {
    // bold persists around inline code: **bold `code` more bold**
    let adf = json!({
        "type": "doc", "version": 1,
        "content": [{
            "type": "paragraph",
            "content": [
                { "type": "text", "text": "bold ", "marks": [{ "type": "strong" }] },
                { "type": "text", "text": "code", "marks": [{ "type": "strong" }, { "type": "code" }] },
                { "type": "text", "text": " more bold", "marks": [{ "type": "strong" }] }
            ]
        }]
    });
    assert_eq!(adf_to_markdown(&adf), "**bold `code` more bold**");
}

#[test]
fn stateful_marks_italic_wraps_bold() {
    // italic outer, bold inner: _italic **bold** italic_
    let adf = json!({
        "type": "doc", "version": 1,
        "content": [{
            "type": "paragraph",
            "content": [
                { "type": "text", "text": "italic ", "marks": [{ "type": "em" }] },
                { "type": "text", "text": "bold", "marks": [{ "type": "em" }, { "type": "strong" }] },
                { "type": "text", "text": " italic", "marks": [{ "type": "em" }] }
            ]
        }]
    });
    assert_eq!(adf_to_markdown(&adf), "_italic **bold** italic_");
}

#[test]
fn nested_bullet_list_markdown_to_adf() {
    let md = "- parent\n  - child1\n  - child2\n- sibling";
    let adf = markdown_to_adf(md);
    let content = adf["content"].as_array().expect("doc content array");
    assert_eq!(content.len(), 1);
    let list = &content[0];
    assert_eq!(list["type"], "bulletList");
    let items = list["content"].as_array().expect("list content array");
    // Two top-level items: "parent" (with nested list) and "sibling"
    assert_eq!(items.len(), 2);
    // First item content: paragraph + nested bulletList
    let first_item_content = items[0]["content"]
        .as_array()
        .expect("first list item content array");
    assert_eq!(first_item_content.len(), 2);
    assert_eq!(first_item_content[0]["type"], "paragraph");
    assert_eq!(first_item_content[1]["type"], "bulletList");
    let nested = first_item_content[1]["content"]
        .as_array()
        .expect("nested list content array");
    assert_eq!(nested.len(), 2);
    assert_eq!(nested[0]["content"][0]["content"][0]["text"], "child1");
    assert_eq!(nested[1]["content"][0]["content"][0]["text"], "child2");
    // Second item
    assert_eq!(items[1]["content"][0]["content"][0]["text"], "sibling");
}

#[test]
fn nested_ordered_in_bullet_markdown_to_adf() {
    let md = "- parent\n  1. first\n  2. second";
    let adf = markdown_to_adf(md);
    let list = &adf["content"][0];
    assert_eq!(list["type"], "bulletList");
    let first_item_content = list["content"][0]["content"]
        .as_array()
        .expect("first list item content array");
    // paragraph + nested orderedList
    assert_eq!(first_item_content.len(), 2);
    assert_eq!(first_item_content[1]["type"], "orderedList");
    let nested = first_item_content[1]["content"]
        .as_array()
        .expect("nested list content array");
    assert_eq!(nested.len(), 2);
}

#[test]
fn indented_list_continuation_after_nested_list_markdown_to_adf() {
    let md = "- parent\n  1. first\n  2. second\n  continuation";
    let adf = markdown_to_adf(md);
    let list = &adf["content"][0];
    assert_eq!(list["type"], "bulletList");
    let first_item_content = list["content"][0]["content"]
        .as_array()
        .expect("first list item content array");
    assert_eq!(first_item_content.len(), 2);
    assert_eq!(first_item_content[0]["type"], "paragraph");
    assert_eq!(first_item_content[1]["type"], "orderedList");
    let nested_items = first_item_content[1]["content"]
        .as_array()
        .expect("nested list content array");
    assert_eq!(nested_items.len(), 2);
    let second_item_paragraph = &nested_items[1]["content"][0];
    assert_eq!(second_item_paragraph["content"][0]["text"], "second");
    assert_eq!(second_item_paragraph["content"][1]["text"], " ");
    assert_eq!(second_item_paragraph["content"][2]["text"], "continuation");
}

#[test]
fn code_mark_exclusivity_in_nested_inline_markdown() {
    // Confirmed broken patterns: bold/italic/strike wrapping inline code
    // After the fix these must produce valid ADF (no code+other mark combos).

    // strong wrapping code: **see `config.yaml`**
    let adf = markdown_to_adf("**see `config.yaml`**");
    let content = &adf["content"][0]["content"];
    assert_eq!(content[0]["text"], "see ", "strong-prefix text");
    assert_eq!(content[0]["marks"], json!([{"type": "strong"}]));
    assert_eq!(content[1]["text"], "config.yaml", "code span text");
    assert_eq!(
        content[1]["marks"],
        json!([{"type": "code"}]),
        "code span must have only code mark"
    );
    assert!(
        targeted_schema_errors(&adf).is_empty(),
        "strong+code: {:#?}",
        targeted_schema_errors(&adf)
    );

    // em wrapping code: *run `make test`*
    let adf = markdown_to_adf("*run `make test`*");
    let content = &adf["content"][0]["content"];
    assert_eq!(
        content[1]["marks"],
        json!([{"type": "code"}]),
        "em+code: code span must keep only code mark"
    );
    assert!(
        targeted_schema_errors(&adf).is_empty(),
        "em+code: {:#?}",
        targeted_schema_errors(&adf)
    );

    // strike wrapping code: ~~old `value`~~
    let adf = markdown_to_adf("~~old `value`~~");
    let content = &adf["content"][0]["content"];
    assert_eq!(
        content[1]["marks"],
        json!([{"type": "code"}]),
        "strike+code: code span must keep only code mark"
    );
    assert!(
        targeted_schema_errors(&adf).is_empty(),
        "strike+code: {:#?}",
        targeted_schema_errors(&adf)
    );

    // link containing code: [click `here`](https://example.com)
    let adf = markdown_to_adf("[click `here`](https://example.com)");
    let content = &adf["content"][0]["content"];
    // "click " gets link mark; "here" keeps only code mark (not link)
    assert_eq!(
        content[1]["marks"],
        json!([{"type": "code"}]),
        "link+code: code span must not get link mark"
    );
    assert!(
        targeted_schema_errors(&adf).is_empty(),
        "link+code: {:#?}",
        targeted_schema_errors(&adf)
    );

    // code inside list item: - **fix `foo`**
    let adf = markdown_to_adf("- **fix `foo`**");
    assert!(
        targeted_schema_errors(&adf).is_empty(),
        "list item strong+code: {:#?}",
        targeted_schema_errors(&adf)
    );
    let list_inline = &adf["content"][0]["content"][0]["content"][0]["content"];
    assert_eq!(
        list_inline[1]["marks"],
        json!([{"type": "code"}]),
        "list item: code span must keep only code mark"
    );

    // code inside table cell: | **see `x`** |
    let adf = markdown_to_adf("| **see `x`** |\n| --- |");
    assert!(
        targeted_schema_errors(&adf).is_empty(),
        "table cell strong+code: {:#?}",
        targeted_schema_errors(&adf)
    );

    // code literal text: `**not bold**` — bold delimiters inside backticks are literal text
    let adf = markdown_to_adf("`**not bold**`");
    let content = &adf["content"][0]["content"];
    assert_eq!(
        content[0]["text"], "**not bold**",
        "code span text must be literal"
    );
    assert_eq!(content[0]["marks"], json!([{"type": "code"}]));
    assert!(targeted_schema_errors(&adf).is_empty());
}
