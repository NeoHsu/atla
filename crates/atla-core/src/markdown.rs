use serde_json::{Value, json};

pub fn markdown_to_adf(markdown: &str) -> Value {
    let blocks = parse_markdown_blocks(markdown);
    json!({
        "type": "doc",
        "version": 1,
        "content": blocks,
    })
}

fn parse_markdown_blocks(markdown: &str) -> Vec<Value> {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        if line.trim().is_empty() {
            index += 1;
            continue;
        }

        if let Some((language, code, next_index)) = parse_fenced_code(&lines, index) {
            blocks.push(adf_code_block(&language, &code));
            index = next_index;
            continue;
        }

        if let Some(level) = markdown_heading_level(line) {
            let text = line.trim_start().trim_start_matches('#').trim_start();
            blocks.push(json!({
                "type": "heading",
                "attrs": { "level": level },
                "content": parse_inline_markdown(text),
            }));
            index += 1;
            continue;
        }

        if is_rule(line) {
            blocks.push(json!({ "type": "rule" }));
            index += 1;
            continue;
        }

        if line.trim_start().starts_with('>') {
            let (quote, next_index) = collect_prefixed_block(&lines, index, '>');
            blocks.push(json!({
                "type": "blockquote",
                "content": sanitize_blockquote_content(parse_markdown_blocks(&quote)),
            }));
            index = next_index;
            continue;
        }

        if task_list_marker(line).is_some() {
            let (items, next_index) = collect_task_items(&lines, index);
            blocks.push(json!({
                "type": "taskList",
                "attrs": { "localId": format!("tasklist-{}", index + 1) },
                "content": items,
            }));
            index = next_index;
            continue;
        }

        if unordered_list_text(line).is_some() {
            let (items, next_index) = collect_list_items(&lines, index, false);
            blocks.push(json!({
                "type": "bulletList",
                "content": items,
            }));
            index = next_index;
            continue;
        }

        if ordered_list_text(line).is_some() {
            let order = ordered_list_order(line).unwrap_or(1);
            let (items, next_index) = collect_list_items(&lines, index, true);
            blocks.push(json!({
                "type": "orderedList",
                "attrs": { "order": order },
                "content": items,
            }));
            index = next_index;
            continue;
        }

        if is_table_start(&lines, index) {
            let (table, next_index) = parse_markdown_table(&lines, index);
            blocks.push(table);
            index = next_index;
            continue;
        }

        let (paragraph, next_index) = collect_paragraph(&lines, index);
        blocks.push(adf_paragraph(&paragraph));
        index = next_index;
    }

    blocks
}

pub fn adf_to_markdown(adf: &Value) -> String {
    trim_blank_lines(&render_block(adf, 0))
}

fn render_block(value: &Value, depth: usize) -> String {
    match value {
        Value::Array(items) => render_blocks(items, depth),
        Value::Object(object) => match node_type(object) {
            "doc" => render_blocks(content_items(object), depth),
            "paragraph" => format!("{}\n\n", render_inlines(content_items(object))),
            "heading" => {
                let level = attrs_u64(object, "level").unwrap_or(1).clamp(1, 6);
                format!(
                    "{} {}\n\n",
                    "#".repeat(level as usize),
                    render_inlines(content_items(object))
                )
            }
            "bulletList" => render_list(content_items(object), depth, false, 1),
            "orderedList" => render_list(
                content_items(object),
                depth,
                true,
                attrs_u64(object, "order").unwrap_or(1),
            ),
            "blockquote" => render_blockquote(content_items(object), depth),
            "rule" => "---\n\n".to_owned(),
            "codeBlock" => render_code_block(object),
            "panel" => render_panel(object, depth),
            "table" => render_table(content_items(object)),
            "mediaSingle" | "mediaGroup" => render_blocks(content_items(object), depth),
            "blockCard" | "embedCard" => render_card(object),
            "taskList" | "decisionList" => render_blocks(content_items(object), depth),
            "taskItem" => render_task_item(object, depth),
            "decisionItem" => format!("- {}\n", render_inlines(content_items(object))),
            "expand" | "nestedExpand" => render_expand(object, depth),
            _ => {
                if let Some(content) = object.get("content").and_then(Value::as_array) {
                    render_blocks(content, depth)
                } else {
                    render_inline(value)
                }
            }
        },
        _ => String::new(),
    }
}

fn parse_fenced_code(lines: &[&str], start: usize) -> Option<(String, String, usize)> {
    let line = lines[start].trim_start();
    let fence = if line.starts_with("```") {
        "```"
    } else if line.starts_with("~~~") {
        "~~~"
    } else {
        return None;
    };
    let language = line.trim_start_matches(fence).trim().to_owned();
    let mut code = Vec::new();
    let mut index = start + 1;
    while index < lines.len() {
        if lines[index].trim_start().starts_with(fence) {
            return Some((language, code.join("\n"), index + 1));
        }
        code.push(lines[index]);
        index += 1;
    }

    Some((language, code.join("\n"), index))
}

fn markdown_heading_level(line: &str) -> Option<u64> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if (1..=6).contains(&level) && trimmed.chars().nth(level) == Some(' ') {
        Some(level as u64)
    } else {
        None
    }
}

fn is_rule(line: &str) -> bool {
    let trimmed = line.trim();
    matches!(trimmed, "---" | "***" | "___")
}

fn collect_prefixed_block(lines: &[&str], start: usize, prefix: char) -> (String, usize) {
    let mut collected = Vec::new();
    let mut index = start;
    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if !trimmed.starts_with(prefix) {
            break;
        }
        collected.push(trimmed[1..].trim_start());
        index += 1;
    }
    (collected.join("\n\n"), index)
}

fn collect_task_items(lines: &[&str], start: usize) -> (Vec<Value>, usize) {
    let mut items = Vec::new();
    let mut index = start;
    while index < lines.len() {
        let Some((checked, text)) = task_list_marker(lines[index]) else {
            break;
        };
        items.push(json!({
            "type": "taskItem",
            "attrs": {
                "localId": format!("task-{}", index + 1),
                "state": if checked { "DONE" } else { "TODO" },
            },
            "content": parse_inline_markdown(text),
        }));
        index += 1;
    }
    (items, index)
}

fn list_indent(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

fn is_ordered_list_line(line: &str) -> bool {
    ordered_list_text(line).is_some()
}

fn collect_list_items(lines: &[&str], start: usize, ordered: bool) -> (Vec<Value>, usize) {
    if start >= lines.len() {
        return (Vec::new(), start);
    }
    let base_indent = list_indent(lines[start]);
    let mut items: Vec<Value> = Vec::new();
    let mut index = start;

    while index < lines.len() {
        let line = lines[index];

        // Blank lines always end the list at this level.
        if line.trim().is_empty() {
            break;
        }

        let indent = list_indent(line);

        // Dedented past our base — we're done.
        if indent < base_indent {
            break;
        }

        // More indented than base by ≥2 spaces → nested list belonging to the last item.
        if indent >= base_indent + 2 {
            if let Some(last_item) = items.last_mut() {
                let nested_ordered = is_ordered_list_line(line);
                let (sub_items, consumed) = collect_list_items(lines, index, nested_ordered);
                if !sub_items.is_empty() {
                    let order = ordered_list_order(lines[index]).unwrap_or(1);
                    let nested = if nested_ordered {
                        json!({"type": "orderedList", "attrs": {"order": order}, "content": sub_items})
                    } else {
                        json!({"type": "bulletList", "content": sub_items})
                    };
                    if let Some(content) =
                        last_item.get_mut("content").and_then(Value::as_array_mut)
                    {
                        content.push(nested);
                    }
                }
                index = consumed;
            } else {
                break; // nested without a parent — malformed input
            }
            continue;
        }

        // Same indent level: must match the expected list type.
        let text = if ordered {
            ordered_list_text(line)
        } else {
            unordered_list_text(line)
        };
        let Some(text) = text else { break };

        items.push(json!({
            "type": "listItem",
            "content": [adf_paragraph(&[(text.to_owned(), false)])],
        }));
        index += 1;
    }

    (items, index)
}

fn collect_paragraph(lines: &[&str], start: usize) -> (Vec<(String, bool)>, usize) {
    let mut collected: Vec<(String, bool)> = Vec::new();
    let mut index = start;
    while index < lines.len() {
        let line = lines[index];
        if line.trim().is_empty()
            || parse_fenced_code(lines, index).is_some()
            || markdown_heading_level(line).is_some()
            || is_rule(line)
            || line.trim_start().starts_with('>')
            || task_list_marker(line).is_some()
            || unordered_list_text(line).is_some()
            || ordered_list_text(line).is_some()
            || is_table_start(lines, index)
        {
            break;
        }
        // Detect hard line break: trailing two or more spaces.
        let hard_break = line.ends_with("  ") || line.ends_with('\t');
        collected.push((line.trim().to_owned(), hard_break));
        index += 1;
    }
    (collected, index)
}

fn is_table_start(lines: &[&str], index: usize) -> bool {
    if index + 1 >= lines.len() {
        return false;
    }
    let line = lines[index];
    let next = lines[index + 1];
    is_table_row(line) && is_table_separator(next)
}

fn is_table_row(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('|') && t.contains('|')
}

fn is_table_separator(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('|')
        && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'))
        && t.contains('-')
}

fn parse_table_row_cells(line: &str) -> Vec<String> {
    let t = line.trim().trim_matches('|');
    t.split('|').map(|cell| cell.trim().to_owned()).collect()
}

fn adf_table_cell(text: &str, header: bool) -> Value {
    let cell_type = if header { "tableHeader" } else { "tableCell" };
    json!({
        "type": cell_type,
        "attrs": {},
        "content": [{"type": "paragraph", "content": parse_inline_markdown(text)}],
    })
}

fn parse_markdown_table(lines: &[&str], start: usize) -> (Value, usize) {
    let headers = parse_table_row_cells(lines[start]);
    let mut index = start + 2; // skip header + separator
    let mut rows: Vec<Value> = vec![json!({
        "type": "tableRow",
        "content": headers.iter().map(|h| adf_table_cell(h, true)).collect::<Vec<_>>(),
    })];
    while index < lines.len() {
        let line = lines[index];
        if !is_table_row(line) {
            break;
        }
        let cells = parse_table_row_cells(line);
        rows.push(json!({
            "type": "tableRow",
            "content": cells.iter().map(|c| adf_table_cell(c, false)).collect::<Vec<_>>(),
        }));
        index += 1;
    }
    (
        json!({
            "type": "table",
            "attrs": {"isNumberColumnEnabled": false, "layout": "default"},
            "content": rows,
        }),
        index,
    )
}

fn task_list_marker(line: &str) -> Option<(bool, &str)> {
    let text = unordered_list_text(line)?;
    let trimmed = text.trim_start();
    if trimmed.len() >= 4 && trimmed.as_bytes()[0] == b'[' && trimmed.as_bytes()[2] == b']' {
        let marker = trimmed.as_bytes()[1] as char;
        if marker == ' ' || marker.eq_ignore_ascii_case(&'x') {
            return Some((marker.eq_ignore_ascii_case(&'x'), trimmed[3..].trim_start()));
        }
    }
    None
}

fn unordered_list_text(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next()?;
    if matches!(marker, '-' | '*' | '+') && trimmed.chars().nth(1) == Some(' ') {
        Some(trimmed[2..].trim_start())
    } else {
        None
    }
}

fn ordered_list_text(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let digits = trimmed.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digits > 0
        && trimmed.chars().nth(digits) == Some('.')
        && trimmed.chars().nth(digits + 1) == Some(' ')
    {
        Some(trimmed[digits + 2..].trim_start())
    } else {
        None
    }
}

fn ordered_list_order(line: &str) -> Option<u64> {
    let trimmed = line.trim_start();
    let digits = trimmed.chars().take_while(|ch| ch.is_ascii_digit()).count();
    trimmed[..digits].parse().ok()
}

fn adf_paragraph(lines: &[(String, bool)]) -> Value {
    if lines.is_empty() {
        return json!({"type": "paragraph", "content": []});
    }
    let mut content: Vec<Value> = Vec::new();
    for (i, (line, hard_break)) in lines.iter().enumerate() {
        content.extend(parse_inline_markdown(line));
        if *hard_break && i + 1 < lines.len() {
            content.push(json!({"type": "hardBreak"}));
        } else if !hard_break && i + 1 < lines.len() {
            // Soft wrap: join with a space character
            content.push(json!({"type": "text", "text": " "}));
        }
    }
    json!({"type": "paragraph", "content": content})
}

fn adf_code_block(language: &str, code: &str) -> Value {
    let mut block = json!({ "type": "codeBlock" });
    if !code.is_empty() {
        block["content"] = json!([
            {
                "type": "text",
                "text": code,
            }
        ]);
    }
    if !language.is_empty() {
        block["attrs"] = json!({ "language": language });
    }
    block
}

fn parse_inline_markdown(text: &str) -> Vec<Value> {
    let mut nodes = Vec::new();
    let mut plain = String::new();
    let mut index = 0;

    while index < text.len() {
        let rest = &text[index..];
        if let Some((new_nodes, consumed)) = parse_inline_token(rest) {
            push_plain_text(&mut nodes, &mut plain);
            nodes.extend(new_nodes);
            index += consumed;
        } else if let Some(ch) = rest.chars().next() {
            plain.push(ch);
            index += ch.len_utf8();
        } else {
            break;
        }
    }
    push_plain_text(&mut nodes, &mut plain);
    nodes
}

fn parse_inline_token(text: &str) -> Option<(Vec<Value>, usize)> {
    // Handle backslash escapes: \* \_ \~ \\ \[ \] \! \| \# \`
    if let Some(rest) = text.strip_prefix('\\')
        && let Some(ch) = rest.chars().next()
        && matches!(
            ch,
            '*' | '_' | '~' | '\\' | '[' | ']' | '!' | '|' | '#' | '`'
        )
    {
        return Some((
            vec![json!({"type": "text", "text": ch.to_string()})],
            1 + ch.len_utf8(),
        ));
    }

    if let Some((nodes, consumed)) = parse_delimited_mark(text, "**", "strong") {
        return Some((nodes, consumed));
    }
    if let Some((nodes, consumed)) = parse_delimited_mark(text, "__", "strong") {
        return Some((nodes, consumed));
    }
    if let Some((nodes, consumed)) = parse_delimited_mark(text, "*", "em") {
        return Some((nodes, consumed));
    }
    if let Some((nodes, consumed)) = parse_delimited_mark(text, "_", "em") {
        return Some((nodes, consumed));
    }
    if let Some((nodes, consumed)) = parse_delimited_mark(text, "~~", "strike") {
        return Some((nodes, consumed));
    }
    if let Some(rest) = text.strip_prefix('`') {
        let end = rest.find('`')?;
        let inner = &rest[..end];
        if inner.is_empty() {
            return None;
        }
        return Some((vec![marked_text(inner, "code")], end + 2));
    }
    if let Some(rest) = text.strip_prefix('[') {
        let label_end = rest.find(']')?;
        let after_label = &rest[label_end + 1..];
        if let Some(after_open) = after_label.strip_prefix('(') {
            let url_end = after_open.find(')')?;
            let label = &rest[..label_end];
            let raw_url = &after_open[..url_end];
            let url = strip_link_title(raw_url);
            // 1([) + label_end + 1(]) + 1(() + url_end + 1())
            return Some((link_text(label, url), label_end + url_end + 4));
        }
    }
    if let Some(rest) = text.strip_prefix("![") {
        let alt_end = rest.find(']')?;
        let after_alt = &rest[alt_end + 1..];
        if let Some(after_open) = after_alt.strip_prefix('(') {
            let url_end = after_open.find(')')?;
            let alt = &rest[..alt_end];
            let raw_url = &after_open[..url_end];
            let url = strip_link_title(raw_url);
            // 2(![) + alt_end + 1(]) + 1(() + url_end + 1())
            return Some((vec![inline_card(url, alt)], alt_end + url_end + 5));
        }
    }
    None
}

/// Strip an optional quoted title from a link URL: `url "title"` → `url`.
fn strip_link_title(url: &str) -> &str {
    let url = url.trim();
    // Check for trailing `"title"` or `'title'`
    if let Some(space_pos) = url.rfind(" \"")
        && url.ends_with('"')
    {
        return url[..space_pos].trim();
    }
    if let Some(space_pos) = url.rfind(" '")
        && url.ends_with('\'')
    {
        return url[..space_pos].trim();
    }
    url
}

fn push_plain_text(nodes: &mut Vec<Value>, plain: &mut String) {
    if !plain.is_empty() {
        nodes.push(json!({
            "type": "text",
            "text": std::mem::take(plain),
        }));
    }
}

fn marked_text(text: &str, mark_type: &str) -> Value {
    json!({
        "type": "text",
        "text": text,
        "marks": [
            { "type": mark_type }
        ],
    })
}

fn parse_delimited_mark(
    text: &str,
    delimiter: &str,
    mark_type: &str,
) -> Option<(Vec<Value>, usize)> {
    let rest = text.strip_prefix(delimiter)?;
    let end = rest.find(delimiter)?;
    let inner = &rest[..end];
    let mut nodes = parse_inline_markdown(inner);
    if nodes.is_empty() {
        return None;
    }
    add_mark_to_nodes(&mut nodes, &json!({ "type": mark_type }));
    Some((nodes, end + delimiter.len() * 2))
}

fn add_mark_to_nodes(nodes: &mut [Value], mark: &Value) {
    for node in nodes {
        add_mark(node, mark.clone());
    }
}

fn add_mark(node: &mut Value, mark: Value) {
    let Some(object) = node.as_object_mut() else {
        return;
    };
    if object.get("type").and_then(Value::as_str) != Some("text") {
        return;
    }
    // ADF: the `code` mark is exclusive — it cannot coexist with any other mark.
    // Don't add any mark to a node that already has `code`, and don't add `code`
    // to a node that already has other marks.
    let is_code_mark = mark.get("type").and_then(Value::as_str) == Some("code");
    if let Some(Value::Array(existing)) = object.get("marks") {
        if existing
            .iter()
            .any(|m| m.get("type").and_then(Value::as_str) == Some("code"))
        {
            return;
        }
        if is_code_mark && !existing.is_empty() {
            return;
        }
    }
    match object.get_mut("marks") {
        Some(Value::Array(marks)) => marks.push(mark),
        _ => {
            object.insert("marks".to_owned(), Value::Array(vec![mark]));
        }
    }
}

fn link_text(text: &str, url: &str) -> Vec<Value> {
    let mut nodes = parse_inline_markdown(text);
    if nodes.is_empty() {
        nodes.push(json!({
            "type": "text",
            "text": text,
        }));
    }
    add_mark_to_nodes(
        &mut nodes,
        &json!({
            "type": "link",
            "attrs": { "href": url }
        }),
    );
    nodes
}

fn inline_card(url: &str, alt: &str) -> Value {
    if alt.is_empty() {
        json!({
            "type": "inlineCard",
            "attrs": { "url": url },
        })
    } else {
        json!({
            "type": "text",
            "text": alt,
            "marks": [
                {
                    "type": "link",
                    "attrs": { "href": url }
                }
            ],
        })
    }
}

fn sanitize_blockquote_content(blocks: Vec<Value>) -> Vec<Value> {
    blocks.into_iter().map(sanitize_blockquote_block).collect()
}

fn sanitize_blockquote_block(block: Value) -> Value {
    let Value::Object(object) = &block else {
        return block;
    };
    if node_type(object) == "heading" {
        return json!({
            "type": "paragraph",
            "content": content_items(object),
        });
    }
    block
}

fn render_blocks(items: &[Value], depth: usize) -> String {
    items
        .iter()
        .map(|item| render_block(item, depth))
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

fn render_list(items: &[Value], depth: usize, ordered: bool, start: u64) -> String {
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
        out.push_str(&render_list_item(object, depth, &marker));
    }
    out.push('\n');
    out
}

fn render_list_item(object: &serde_json::Map<String, Value>, depth: usize, marker: &str) -> String {
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
                )),
                "orderedList" => nested_blocks.push(render_list(
                    content_items(child_object),
                    depth + 1,
                    true,
                    attrs_u64(child_object, "order").unwrap_or(1),
                )),
                _ => {
                    let rendered = trim_blank_lines(&render_block(child, depth));
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

fn render_blockquote(items: &[Value], depth: usize) -> String {
    let body = trim_blank_lines(&render_blocks(items, depth));
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

fn render_panel(object: &serde_json::Map<String, Value>, depth: usize) -> String {
    let panel_type = attrs_str(object, "panelType").unwrap_or("panel");
    let body = trim_blank_lines(&render_blocks(content_items(object), depth));
    format!("> **{panel_type}:**\n>\n{}\n\n", prefix_lines(&body, "> "))
}

fn render_expand(object: &serde_json::Map<String, Value>, depth: usize) -> String {
    let title = attrs_str(object, "title").unwrap_or("Details");
    let body = trim_blank_lines(&render_blocks(content_items(object), depth));
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

fn render_table(rows: &[Value]) -> String {
    let rows = rows
        .iter()
        .filter_map(|row| match row {
            Value::Object(object) if node_type(object) == "tableRow" => Some(
                content_items(object)
                    .iter()
                    .filter_map(|cell| match cell {
                        Value::Object(cell_object)
                            if matches!(node_type(cell_object), "tableHeader" | "tableCell") =>
                        {
                            Some(table_cell_text(content_items(cell_object)))
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
    out.push_str(&format_table_row(first_row, width));
    out.push_str(&format_table_separator(width));
    for row in rows.iter().skip(1) {
        out.push_str(&format_table_row(row, width));
    }
    out.push('\n');
    out
}

fn table_cell_text(items: &[Value]) -> String {
    trim_blank_lines(&render_blocks(items, 0))
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

fn content_items(object: &serde_json::Map<String, Value>) -> &[Value] {
    object
        .get("content")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn node_type(object: &serde_json::Map<String, Value>) -> &str {
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

fn trim_blank_lines(text: &str) -> String {
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

#[cfg(test)]
mod tests {
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

        // Two separate paragraphs inside the blockquote
        assert_eq!(adf["content"][0]["type"], json!("blockquote"));
        assert_eq!(adf["content"][0]["content"].as_array().unwrap().len(), 2);

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
        let content = adf["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        let list = &content[0];
        assert_eq!(list["type"], "bulletList");
        let items = list["content"].as_array().unwrap();
        // Two top-level items: "parent" (with nested list) and "sibling"
        assert_eq!(items.len(), 2);
        // First item content: paragraph + nested bulletList
        let first_item_content = items[0]["content"].as_array().unwrap();
        assert_eq!(first_item_content.len(), 2);
        assert_eq!(first_item_content[0]["type"], "paragraph");
        assert_eq!(first_item_content[1]["type"], "bulletList");
        let nested = first_item_content[1]["content"].as_array().unwrap();
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
        let first_item_content = list["content"][0]["content"].as_array().unwrap();
        // paragraph + nested orderedList
        assert_eq!(first_item_content.len(), 2);
        assert_eq!(first_item_content[1]["type"], "orderedList");
        let nested = first_item_content[1]["content"].as_array().unwrap();
        assert_eq!(nested.len(), 2);
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
}
