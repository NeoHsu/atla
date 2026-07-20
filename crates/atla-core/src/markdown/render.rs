use super::AdfToMarkdownOptions;
use serde_json::Value;

pub(super) fn render_block(value: &Value, depth: usize, options: AdfToMarkdownOptions) -> String {
    match value {
        Value::Array(items) => render_blocks(items, depth, options),
        Value::Object(object) => match node_type(object) {
            "doc" => render_blocks(content_items(object), depth, options),
            "paragraph" => format!("{}\n\n", render_inlines(content_items(object))),
            "heading" => {
                let level = attrs_u64(object, "level").unwrap_or(1).clamp(1, 6);
                format!(
                    "{} {}\n\n",
                    "#".repeat(level as usize),
                    render_inlines(content_items(object))
                )
            }
            "bulletList" => render_list(content_items(object), depth, false, 1, options),
            "orderedList" => render_list(
                content_items(object),
                depth,
                true,
                attrs_u64(object, "order").unwrap_or(1),
                options,
            ),
            "blockquote" => render_blockquote(content_items(object), depth, options),
            "rule" => "---\n\n".to_owned(),
            "codeBlock" => render_code_block(object),
            "panel" => render_panel(object, depth, options),
            "table" => render_table(object, options),
            "mediaSingle" | "mediaGroup" => render_blocks(content_items(object), depth, options),
            "blockCard" | "embedCard" => render_card(object),
            "taskList" | "decisionList" => render_blocks(content_items(object), depth, options),
            "taskItem" => render_task_item(object, depth),
            "decisionItem" => format!("- {}\n", render_inlines(content_items(object))),
            "expand" | "nestedExpand" => render_expand(object, depth, options),
            _ => {
                if let Some(content) = object.get("content").and_then(Value::as_array) {
                    render_blocks(content, depth, options)
                } else {
                    render_inline(value)
                }
            }
        },
        _ => String::new(),
    }
}

fn render_blocks(items: &[Value], depth: usize, options: AdfToMarkdownOptions) -> String {
    items
        .iter()
        .map(|item| render_block(item, depth, options))
        .collect::<String>()
}

fn render_inline(value: &Value) -> String {
    let Value::Object(object) = value else {
        return String::new();
    };

    let rendered = match node_type(object) {
        "text" => object
            .get("text")
            .and_then(Value::as_str)
            .map(escape_text)
            .unwrap_or_default(),
        "hardBreak" => "  \n".to_owned(),
        "mention" => attrs_str(object, "text")
            .or_else(|| attrs_str(object, "displayName"))
            .or_else(|| attrs_str(object, "id"))
            .map(|text| format!("@{text}"))
            .unwrap_or_else(|| "@mention".to_owned()),
        "emoji" => attrs_str(object, "shortName")
            .or_else(|| attrs_str(object, "text"))
            .unwrap_or(":emoji:")
            .to_owned(),
        "inlineCard" => attrs_str(object, "url")
            .unwrap_or("[inline card]")
            .to_owned(),
        "date" => attrs_str(object, "timestamp")
            .unwrap_or("[date]")
            .to_owned(),
        "status" => attrs_str(object, "text")
            .map(|text| format!("`{text}`"))
            .unwrap_or_else(|| "`status`".to_owned()),
        "media" => render_media(object),
        _ => render_inlines(content_items(object)),
    };

    apply_marks(&rendered, object.get("marks").and_then(Value::as_array))
}

/// Render a sequence of inline nodes with stateful mark tracking so that adjacent text
/// nodes sharing outer marks (e.g. `**bold _italic_**`) do not produce redundant delimiters
/// (e.g. `**bold ****_italic_**`).
///
/// Stateful marks (strong, em, strike, underline) are tracked across text nodes: they are
/// opened/closed only at transitions.  Atomic marks (code, link, subsup) are applied on each
/// individual text node and do not participate in the stateful tracking.
fn render_inlines(items: &[Value]) -> String {
    // Count how many text nodes each stateful mark type appears on.
    // Higher frequency means the mark spans more text → it is likely the outer mark.
    let mut freq: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for item in items {
        if let Value::Object(obj) = item
            && node_type(obj) == "text"
        {
            for mt in stateful_marks_of(obj) {
                *freq.entry(mt).or_default() += 1;
            }
        }
    }

    let mut result = String::new();
    let mut open: Vec<&'static str> = vec![];

    for item in items {
        let Value::Object(obj) = item else { continue };

        if node_type(obj) != "text" {
            // Non-text inline node: close stateful marks, render with old approach, reopen.
            for m in open.iter().rev() {
                result.push_str(stateful_close(m));
            }
            let saved = std::mem::take(&mut open);
            result.push_str(&render_inline(item));
            for m in &saved {
                result.push_str(stateful_open(m));
            }
            open = saved;
            continue;
        }

        // Escape the raw text, then apply atomic marks (code, link, subsup).
        let raw = obj
            .get("text")
            .and_then(Value::as_str)
            .map(escape_text)
            .unwrap_or_default();
        let text = apply_atomic_marks(raw, obj);

        // Compute this node's stateful marks sorted outer-first.
        let node_marks = canonical_stateful_marks(obj, &freq);

        // Transition: close marks no longer active, open newly active marks.
        let common = open
            .iter()
            .zip(node_marks.iter())
            .take_while(|(a, b)| **a == **b)
            .count();
        for m in open[common..].iter().rev() {
            result.push_str(stateful_close(m));
        }
        for m in &node_marks[common..] {
            result.push_str(stateful_open(m));
        }
        open = node_marks;
        result.push_str(&text);
    }

    for m in open.iter().rev() {
        result.push_str(stateful_close(m));
    }
    result
}

/// Returns the stateful mark types present on a text node (strong, em, strike, underline).
fn stateful_marks_of(obj: &serde_json::Map<String, Value>) -> Vec<&'static str> {
    obj.get("marks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(
            |m| match m.get("type").and_then(Value::as_str).unwrap_or_default() {
                "strong" => Some("strong"),
                "em" => Some("em"),
                "strike" => Some("strike"),
                "underline" => Some("underline"),
                _ => None,
            },
        )
        .collect()
}

/// Returns stateful marks sorted outermost-first:
/// 1. Higher frequency across the inline sequence = outer mark.
/// 2. On a tie: higher position in the ADF marks array = outer (because `apply_marks` folds
///    left-to-right, so the last mark in the array is the outermost wrapper).
fn canonical_stateful_marks(
    obj: &serde_json::Map<String, Value>,
    freq: &std::collections::HashMap<&str, usize>,
) -> Vec<&'static str> {
    let mut marks_with_pos: Vec<(&'static str, usize)> = obj
        .get("marks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(
            |(pos, m)| match m.get("type").and_then(Value::as_str).unwrap_or_default() {
                "strong" => Some(("strong", pos)),
                "em" => Some(("em", pos)),
                "strike" => Some(("strike", pos)),
                "underline" => Some(("underline", pos)),
                _ => None,
            },
        )
        .collect();

    marks_with_pos.sort_by(|(a, pos_a), (b, pos_b)| {
        let fa = freq.get(*a).copied().unwrap_or(0);
        let fb = freq.get(*b).copied().unwrap_or(0);
        // Higher frequency = outer (first).  On tie, higher ADF position = outer (first).
        fb.cmp(&fa).then_with(|| pos_b.cmp(pos_a))
    });

    marks_with_pos.into_iter().map(|(m, _)| m).collect()
}

fn stateful_open(mark_type: &str) -> &'static str {
    match mark_type {
        "strong" => "**",
        "em" => "_",
        "strike" => "~~",
        "underline" => "<u>",
        _ => "",
    }
}

fn stateful_close(mark_type: &str) -> &'static str {
    match mark_type {
        "strong" => "**",
        "em" => "_",
        "strike" => "~~",
        "underline" => "</u>",
        _ => "",
    }
}

/// Applies only atomic marks (code, link, subsup) to the text, leaving stateful marks
/// to be handled by the surrounding render_inlines logic.
fn apply_atomic_marks(text: String, obj: &serde_json::Map<String, Value>) -> String {
    let Some(marks) = obj.get("marks").and_then(Value::as_array) else {
        return text;
    };
    marks.iter().fold(text, |current, mark| {
        let Value::Object(m) = mark else {
            return current;
        };
        match m.get("type").and_then(Value::as_str).unwrap_or_default() {
            "code" => format!("`{}`", current.replace('`', "\\`")),
            "link" => m
                .get("attrs")
                .and_then(|a| a.get("href"))
                .and_then(Value::as_str)
                .map(|href| format!("[{current}]({href})"))
                .unwrap_or(current),
            "subsup" => match m
                .get("attrs")
                .and_then(|a| a.get("type"))
                .and_then(Value::as_str)
            {
                Some("sub") => format!("<sub>{current}</sub>"),
                Some("sup") => format!("<sup>{current}</sup>"),
                _ => current,
            },
            _ => current, // stateful marks handled by render_inlines
        }
    })
}

fn render_list(
    items: &[Value],
    depth: usize,
    ordered: bool,
    start: u64,
    options: AdfToMarkdownOptions,
) -> String {
    let mut out = String::new();
    for (index, item) in items.iter().enumerate() {
        let Value::Object(object) = item else {
            continue;
        };
        let marker = if ordered {
            format!("{}.", start + index as u64)
        } else {
            "-".to_owned()
        };
        out.push_str(&render_list_item(object, depth, &marker, options));
    }
    out.push('\n');
    out
}

fn render_list_item(
    object: &serde_json::Map<String, Value>,
    depth: usize,
    marker: &str,
    options: AdfToMarkdownOptions,
) -> String {
    let mut text_parts = Vec::new();
    let mut nested_blocks = Vec::new();

    for child in content_items(object) {
        if let Value::Object(child_object) = child {
            match node_type(child_object) {
                "bulletList" => nested_blocks.push(render_list(
                    content_items(child_object),
                    depth + 1,
                    false,
                    1,
                    options,
                )),
                "orderedList" => nested_blocks.push(render_list(
                    content_items(child_object),
                    depth + 1,
                    true,
                    attrs_u64(child_object, "order").unwrap_or(1),
                    options,
                )),
                _ => {
                    let rendered = trim_blank_lines(&render_block(child, depth, options));
                    if !rendered.is_empty() {
                        text_parts.push(rendered);
                    }
                }
            }
        }
    }

    let indent = "  ".repeat(depth);
    let continuation_indent = " ".repeat(marker.len() + 1);
    let text = text_parts.join("\n");
    let mut lines = text.lines();
    let first = lines.next().unwrap_or_default();
    let mut out = format!("{indent}{marker} {first}\n");
    for line in lines {
        out.push_str(&indent);
        out.push_str(&continuation_indent);
        out.push_str(line);
        out.push('\n');
    }
    for nested in nested_blocks {
        out.push_str(nested.trim_end_matches('\n'));
        out.push('\n');
    }
    out
}

fn render_blockquote(items: &[Value], depth: usize, options: AdfToMarkdownOptions) -> String {
    let body = trim_blank_lines(&render_blocks(items, depth, options));
    let quoted = body
        .lines()
        .map(|line| {
            if line.is_empty() {
                ">".to_owned()
            } else {
                format!("> {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{quoted}\n\n")
}

fn render_code_block(object: &serde_json::Map<String, Value>) -> String {
    let language = attrs_str(object, "language").unwrap_or_default();
    let code = collect_plain_text(content_items(object));
    format!("```{language}\n{}\n```\n\n", code.trim_end())
}

fn render_panel(
    object: &serde_json::Map<String, Value>,
    depth: usize,
    options: AdfToMarkdownOptions,
) -> String {
    let panel_type = attrs_str(object, "panelType").unwrap_or("panel");
    let body = trim_blank_lines(&render_blocks(content_items(object), depth, options));
    format!("> **{panel_type}:**\n>\n{}\n\n", prefix_lines(&body, "> "))
}

fn render_expand(
    object: &serde_json::Map<String, Value>,
    depth: usize,
    options: AdfToMarkdownOptions,
) -> String {
    let title = attrs_str(object, "title").unwrap_or("Details");
    let body = trim_blank_lines(&render_blocks(content_items(object), depth, options));
    format!("<details>\n<summary>{title}</summary>\n\n{body}\n\n</details>\n\n")
}

fn render_task_item(object: &serde_json::Map<String, Value>, _depth: usize) -> String {
    let checked = match attrs_str(object, "state") {
        Some("DONE") | Some("done") | Some("checked") => "x",
        _ => " ",
    };
    format!("- [{checked}] {}\n", render_inlines(content_items(object)))
}

fn render_card(object: &serde_json::Map<String, Value>) -> String {
    attrs_str(object, "url")
        .map(|url| format!("{url}\n\n"))
        .unwrap_or_default()
}

fn render_media(object: &serde_json::Map<String, Value>) -> String {
    let alt = attrs_str(object, "alt")
        .or_else(|| attrs_str(object, "id"))
        .unwrap_or("media");
    attrs_str(object, "url")
        .map(|url| format!("![{alt}]({url})"))
        .unwrap_or_else(|| format!("[media: {alt}]"))
}

fn render_table(object: &serde_json::Map<String, Value>, options: AdfToMarkdownOptions) -> String {
    let rows = content_items(object)
        .iter()
        .filter_map(|row| match row {
            Value::Object(object) if node_type(object) == "tableRow" => Some(
                content_items(object)
                    .iter()
                    .filter_map(|cell| match cell {
                        Value::Object(cell_object)
                            if matches!(node_type(cell_object), "tableHeader" | "tableCell") =>
                        {
                            Some(table_cell_text(content_items(cell_object), options))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();

    let Some(first_row) = rows.first() else {
        return String::new();
    };
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = String::new();
    if options.table_numbered_rows_directives
        && attrs_bool(object, "isNumberColumnEnabled").unwrap_or(false)
    {
        out.push_str("<!-- atla:table numbered-rows=true -->\n");
    }
    out.push_str(&format_table_row(first_row, width));
    out.push_str(&format_table_separator(width));
    for row in rows.iter().skip(1) {
        out.push_str(&format_table_row(row, width));
    }
    out.push('\n');
    out
}

fn table_cell_text(items: &[Value], options: AdfToMarkdownOptions) -> String {
    trim_blank_lines(&render_blocks(items, 0, options))
        .replace('\n', "<br>")
        .replace('|', "\\|")
}

fn format_table_row(row: &[String], width: usize) -> String {
    let mut cells = row.to_vec();
    cells.resize(width, String::new());
    format!("| {} |\n", cells.join(" | "))
}

fn format_table_separator(width: usize) -> String {
    format!(
        "|{}|\n",
        (0..width).map(|_| " --- ").collect::<Vec<_>>().join("|")
    )
}

fn apply_marks(text: &str, marks: Option<&Vec<Value>>) -> String {
    let Some(marks) = marks else {
        return text.to_owned();
    };
    marks.iter().fold(text.to_owned(), |current, mark| {
        let Value::Object(mark) = mark else {
            return current;
        };
        match mark.get("type").and_then(Value::as_str).unwrap_or_default() {
            "strong" => format!("**{current}**"),
            "em" => format!("_{current}_"),
            "strike" => format!("~~{current}~~"),
            "code" => format!("`{}`", current.replace('`', "\\`")),
            "underline" => format!("<u>{current}</u>"),
            "link" => mark
                .get("attrs")
                .and_then(|attrs| attrs.get("href"))
                .and_then(Value::as_str)
                .map(|href| format!("[{current}]({href})"))
                .unwrap_or(current),
            "subsup" => match mark
                .get("attrs")
                .and_then(|attrs| attrs.get("type"))
                .and_then(Value::as_str)
            {
                Some("sub") => format!("<sub>{current}</sub>"),
                Some("sup") => format!("<sup>{current}</sup>"),
                _ => current,
            },
            _ => current,
        }
    })
}

fn collect_plain_text(items: &[Value]) -> String {
    items
        .iter()
        .map(|item| match item {
            Value::Object(object) if node_type(object) == "text" => object
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            Value::Object(object) => collect_plain_text(content_items(object)),
            Value::Array(items) => collect_plain_text(items),
            _ => String::new(),
        })
        .collect::<String>()
}

pub(super) fn content_items(object: &serde_json::Map<String, Value>) -> &[Value] {
    object
        .get("content")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub(super) fn node_type(object: &serde_json::Map<String, Value>) -> &str {
    object
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

fn attrs_str<'a>(object: &'a serde_json::Map<String, Value>, name: &str) -> Option<&'a str> {
    object
        .get("attrs")
        .and_then(|attrs| attrs.get(name))
        .and_then(Value::as_str)
}

fn attrs_bool(object: &serde_json::Map<String, Value>, name: &str) -> Option<bool> {
    object
        .get("attrs")
        .and_then(|attrs| attrs.get(name))
        .and_then(Value::as_bool)
}

fn attrs_u64(object: &serde_json::Map<String, Value>, name: &str) -> Option<u64> {
    object
        .get("attrs")
        .and_then(|attrs| attrs.get(name))
        .and_then(Value::as_u64)
}

fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 4);
    for ch in text.chars() {
        match ch {
            '\\' | '*' | '_' | '~' | '`' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

pub(super) fn trim_blank_lines(text: &str) -> String {
    text.trim_matches(|c| c == '\n' || c == ' ').to_owned()
}

fn prefix_lines(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                prefix.trim_end().to_owned()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
