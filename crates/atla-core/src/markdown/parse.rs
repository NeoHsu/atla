use super::render::{render_block, trim_blank_lines};
use super::{AdfToMarkdownOptions, MarkdownMention, MarkdownToAdfOptions};
use comrak::nodes::{AlertType, AstNode, ListType, NodeValue};
use comrak::{Arena, Options, parse_document};
use serde_json::{Value, json};

pub(super) fn parse_markdown_blocks(markdown: &str, options: &MarkdownToAdfOptions) -> Vec<Value> {
    let arena = Arena::new();
    let comrak_options = markdown_parse_options();
    let root = parse_document(&arena, markdown, &comrak_options);
    MarkdownAdfConverter::new(options).convert_document(root)
}

fn markdown_parse_options() -> Options<'static> {
    let mut options = Options::default();

    // CommonMark is always enabled by comrak. These switches enable the parts of
    // GitHub Flavored Markdown that Confluence users most commonly paste into
    // atla: tables, task lists, strikethrough, and autolinks. A few comrak
    // extensions are enabled when they have a conservative ADF fallback below.
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.tagfilter = true;
    options.extension.footnotes = true;
    options.extension.inline_footnotes = true;
    options.extension.description_lists = true;
    options.extension.multiline_block_quotes = true;
    options.extension.alerts = true;
    options.extension.math_dollars = true;
    options.extension.math_code = true;
    options.extension.wikilinks_title_after_pipe = true;
    options.extension.wikilinks_title_before_pipe = true;
    options.extension.cjk_friendly_emphasis = true;

    options.parse.tasklist_in_table = true;
    options.parse.relaxed_autolinks = true;
    options.parse.leave_footnote_definitions = true;

    options
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockContext {
    Document,
    Blockquote,
    ListItem,
}

struct MarkdownAdfConverter<'options> {
    options: &'options MarkdownToAdfOptions,
    pending_table_numbered_rows: Option<bool>,
    generated_local_id: usize,
}

impl<'options> MarkdownAdfConverter<'options> {
    fn new(options: &'options MarkdownToAdfOptions) -> Self {
        Self {
            options,
            pending_table_numbered_rows: None,
            generated_local_id: 0,
        }
    }

    fn convert_document<'arena>(&mut self, root: &'arena AstNode<'arena>) -> Vec<Value> {
        self.convert_block_children(root, BlockContext::Document)
    }

    fn convert_block_children<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        context: BlockContext,
    ) -> Vec<Value> {
        let mut blocks = Vec::new();

        for child in node.children() {
            if let Some(numbered_rows) = table_numbered_rows_directive_from_node(child) {
                self.pending_table_numbered_rows = Some(numbered_rows);
                continue;
            }

            if !is_table_node(child) {
                self.pending_table_numbered_rows = None;
            }

            blocks.extend(self.convert_block(child, context));
        }

        blocks
    }

    fn convert_block<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        context: BlockContext,
    ) -> Vec<Value> {
        let value = node.data().value.clone();
        match value {
            NodeValue::Document => self.convert_block_children(node, context),
            NodeValue::Paragraph => vec![
                self.paragraph_from_node_with_softbreak(node, context == BlockContext::Blockquote),
            ],
            NodeValue::Heading(heading) => {
                let content = self.convert_inlines(node, &[]);
                if context == BlockContext::Document {
                    vec![json!({
                        "type": "heading",
                        "attrs": { "level": u64::from(heading.level).clamp(1, 6) },
                        "content": content,
                    })]
                } else {
                    vec![json!({ "type": "paragraph", "content": content })]
                }
            }
            NodeValue::ThematicBreak => vec![json!({ "type": "rule" })],
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                let content = sanitize_blockquote_adf_blocks(
                    self.convert_block_children(node, BlockContext::Blockquote),
                );
                vec![json!({ "type": "blockquote", "content": content })]
            }
            NodeValue::List(list) => self.convert_list(node, list),
            NodeValue::Item(_) => vec![self.list_item_from_node(node)],
            NodeValue::TaskItem(task) => {
                vec![self.task_item_from_node(node, task.symbol.is_some())]
            }
            NodeValue::CodeBlock(code) => {
                let language = code.info.split_whitespace().next().unwrap_or_default();
                let code = code.literal.trim_end_matches('\n');
                vec![adf_code_block(language, code)]
            }
            NodeValue::HtmlBlock(html) => {
                let html = html.literal.trim_end_matches('\n');
                if html.trim().is_empty() {
                    Vec::new()
                } else {
                    vec![adf_code_block("html", html)]
                }
            }
            NodeValue::Table(_) if context == BlockContext::Blockquote => {
                self.fallback_block_as_paragraph(node)
            }
            NodeValue::Table(_) => self.convert_table(node),
            NodeValue::TableRow(_) | NodeValue::TableCell => self.fallback_block_as_paragraph(node),
            NodeValue::FootnoteDefinition(footnote) => {
                let mut body = String::new();
                append_plain_text_for_children(node, &mut body);
                let body = body.trim();
                let text = if body.is_empty() {
                    format!("[^{}]:", footnote.name)
                } else {
                    format!("[^{}]: {body}", footnote.name)
                };
                vec![self.paragraph_from_text(&text)]
            }
            NodeValue::DescriptionList => self.convert_description_list(node),
            NodeValue::DescriptionItem(_)
            | NodeValue::DescriptionTerm
            | NodeValue::DescriptionDetails => self.fallback_block_as_paragraph(node),
            NodeValue::Alert(alert) => {
                let content = self.convert_block_children(node, context);
                if context == BlockContext::Blockquote {
                    sanitize_blockquote_adf_blocks(content)
                } else {
                    vec![json!({
                        "type": "panel",
                        "attrs": { "panelType": panel_type_for_alert(alert.alert_type) },
                        "content": content,
                    })]
                }
            }
            NodeValue::BlockDirective(directive) => {
                let content = self.convert_block_children(node, context);
                if context == BlockContext::Blockquote {
                    sanitize_blockquote_adf_blocks(content)
                } else {
                    vec![json!({
                        "type": "panel",
                        "attrs": { "panelType": panel_type_for_directive(&directive.info) },
                        "content": content,
                    })]
                }
            }
            NodeValue::Subtext => {
                let mark = json!({"type": "subsup", "attrs": {"type": "sub"}});
                vec![json!({
                    "type": "paragraph",
                    "content": self.convert_inlines(node, &[mark]),
                })]
            }
            NodeValue::FrontMatter(front_matter) => vec![adf_code_block("yaml", &front_matter)],
            value if value.block() => {
                let blocks = self.convert_block_children(node, context);
                if blocks.is_empty() {
                    self.fallback_block_as_paragraph(node)
                } else {
                    blocks
                }
            }
            _ => {
                let content = self.convert_inline(node, &[], false);
                if content.is_empty() {
                    Vec::new()
                } else {
                    vec![json!({ "type": "paragraph", "content": content })]
                }
            }
        }
    }

    fn convert_list<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        list: comrak::nodes::NodeList,
    ) -> Vec<Value> {
        let children = node.children().collect::<Vec<_>>();
        let all_task_items = !children.is_empty()
            && children
                .iter()
                .all(|child| matches!(child.data().value, NodeValue::TaskItem(_)));

        if list.is_task_list && all_task_items {
            let items = children
                .into_iter()
                .filter_map(|child| match child.data().value.clone() {
                    NodeValue::TaskItem(task) => {
                        Some(self.task_item_from_node(child, task.symbol.is_some()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            return vec![json!({
                "type": "taskList",
                "attrs": { "localId": self.local_id("tasklist", node) },
                "content": items,
            })];
        }

        let items = children
            .into_iter()
            .filter_map(|child| match child.data().value.clone() {
                NodeValue::Item(_) => Some(self.list_item_from_node(child)),
                NodeValue::TaskItem(task) => {
                    Some(self.task_item_as_list_item(child, task.symbol.is_some()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        if list.list_type == ListType::Ordered {
            vec![json!({
                "type": "orderedList",
                "attrs": { "order": list.start.max(1) },
                "content": items,
            })]
        } else {
            vec![json!({ "type": "bulletList", "content": items })]
        }
    }

    fn list_item_from_node<'arena>(&mut self, node: &'arena AstNode<'arena>) -> Value {
        let mut content = self.convert_block_children(node, BlockContext::ListItem);
        if content.is_empty() {
            content.push(json!({ "type": "paragraph", "content": [] }));
        }
        json!({ "type": "listItem", "content": content })
    }

    fn task_item_as_list_item<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        checked: bool,
    ) -> Value {
        let mut content = self.convert_task_item_inline_content(node);
        let prefix = if checked { "[x] " } else { "[ ] " };
        content.insert(0, adf_text(prefix, &[]));
        json!({
            "type": "listItem",
            "content": [json!({ "type": "paragraph", "content": content })],
        })
    }

    fn task_item_from_node<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        checked: bool,
    ) -> Value {
        json!({
            "type": "taskItem",
            "attrs": {
                "localId": self.local_id("task", node),
                "state": if checked { "DONE" } else { "TODO" },
            },
            "content": self.convert_task_item_inline_content(node),
        })
    }

    fn convert_task_item_inline_content<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
    ) -> Vec<Value> {
        let mut content = Vec::new();
        for child in node.children() {
            if !content.is_empty() {
                content.push(json!({"type": "hardBreak"}));
            }

            match child.data().value.clone() {
                NodeValue::Paragraph => content.extend(self.convert_inlines(child, &[])),
                _ => {
                    let text = plain_text_for_node(child).trim().to_owned();
                    content.extend(self.text_nodes(&text, &[]));
                }
            }
        }
        content
    }

    fn convert_table<'arena>(&mut self, node: &'arena AstNode<'arena>) -> Vec<Value> {
        let numbered_rows = self.options.numbered_table_rows
            || self.pending_table_numbered_rows.take().unwrap_or(false);
        let rows = node
            .children()
            .filter_map(|row| match row.data().value.clone() {
                NodeValue::TableRow(header) => Some(json!({
                    "type": "tableRow",
                    "content": row
                        .children()
                        .map(|cell| self.table_cell_from_node(cell, header))
                        .collect::<Vec<_>>(),
                })),
                _ => None,
            })
            .collect::<Vec<_>>();

        vec![json!({
            "type": "table",
            "attrs": {"isNumberColumnEnabled": numbered_rows, "layout": "default"},
            "content": rows,
        })]
    }

    fn table_cell_from_node<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        header: bool,
    ) -> Value {
        let cell_type = if header { "tableHeader" } else { "tableCell" };
        let content = if node
            .children()
            .any(|child| matches!(child.data().value, NodeValue::TaskItem(_)))
        {
            let text = plain_text_for_node(node);
            self.text_nodes(text.trim(), &[])
        } else {
            self.convert_inlines(node, &[])
        };

        json!({
            "type": cell_type,
            "attrs": {},
            "content": [{"type": "paragraph", "content": content}],
        })
    }

    fn convert_description_list<'arena>(&mut self, node: &'arena AstNode<'arena>) -> Vec<Value> {
        let items = node
            .children()
            .map(|item| {
                let mut content = self.convert_block_children(item, BlockContext::ListItem);
                if content.is_empty() {
                    content.push(self.paragraph_from_text(plain_text_for_node(item).trim()));
                }
                json!({ "type": "listItem", "content": content })
            })
            .collect::<Vec<_>>();

        vec![json!({ "type": "bulletList", "content": items })]
    }

    fn paragraph_from_node_with_softbreak<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        softbreak_as_hard: bool,
    ) -> Value {
        json!({
            "type": "paragraph",
            "content": self.convert_inlines_with_softbreak(node, &[], softbreak_as_hard),
        })
    }

    fn paragraph_from_text(&self, text: &str) -> Value {
        json!({ "type": "paragraph", "content": self.text_nodes(text, &[]) })
    }

    fn fallback_block_as_paragraph<'arena>(&mut self, node: &'arena AstNode<'arena>) -> Vec<Value> {
        let text = plain_text_for_node(node).trim().to_owned();
        if text.is_empty() {
            Vec::new()
        } else {
            vec![self.paragraph_from_text(&text)]
        }
    }

    fn convert_inlines<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        marks: &[Value],
    ) -> Vec<Value> {
        self.convert_inlines_with_softbreak(node, marks, false)
    }

    fn convert_inlines_with_softbreak<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        marks: &[Value],
        softbreak_as_hard: bool,
    ) -> Vec<Value> {
        let mut content = Vec::new();
        for child in node.children() {
            content.extend(self.convert_inline(child, marks, softbreak_as_hard));
        }
        content
    }

    fn convert_inline<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        marks: &[Value],
        softbreak_as_hard: bool,
    ) -> Vec<Value> {
        let value = node.data().value.clone();
        match value {
            NodeValue::Text(text) => self.text_nodes(text.as_ref(), marks),
            NodeValue::SoftBreak if softbreak_as_hard => vec![json!({"type": "hardBreak"})],
            NodeValue::SoftBreak => vec![adf_text(" ", marks)],
            NodeValue::LineBreak => vec![json!({"type": "hardBreak"})],
            NodeValue::Code(code) => vec![adf_text(&code.literal, &[json!({"type": "code"})])],
            NodeValue::HtmlInline(html) | NodeValue::Raw(html) => self.text_nodes(&html, marks),
            NodeValue::Emph => {
                self.convert_with_mark(node, marks, json!({"type": "em"}), softbreak_as_hard)
            }
            NodeValue::Strong => {
                self.convert_with_mark(node, marks, json!({"type": "strong"}), softbreak_as_hard)
            }
            NodeValue::Strikethrough => {
                self.convert_with_mark(node, marks, json!({"type": "strike"}), softbreak_as_hard)
            }
            NodeValue::Superscript => self.convert_with_mark(
                node,
                marks,
                json!({"type": "subsup", "attrs": {"type": "sup"}}),
                softbreak_as_hard,
            ),
            NodeValue::Subscript => self.convert_with_mark(
                node,
                marks,
                json!({"type": "subsup", "attrs": {"type": "sub"}}),
                softbreak_as_hard,
            ),
            NodeValue::Underline => {
                self.convert_with_mark(node, marks, json!({"type": "underline"}), softbreak_as_hard)
            }
            NodeValue::Insert => {
                self.convert_with_mark(node, marks, json!({"type": "underline"}), softbreak_as_hard)
            }
            NodeValue::Highlight => self.convert_with_mark(
                node,
                marks,
                json!({"type": "backgroundColor", "attrs": {"color": "#FFF0B3"}}),
                softbreak_as_hard,
            ),
            NodeValue::SpoileredText => self.convert_with_mark(
                node,
                marks,
                json!({"type": "backgroundColor", "attrs": {"color": "#C1C7D0"}}),
                softbreak_as_hard,
            ),
            NodeValue::Link(link) => self.convert_with_mark(
                node,
                marks,
                json!({"type": "link", "attrs": {"href": link.url}}),
                softbreak_as_hard,
            ),
            NodeValue::WikiLink(link) => self.convert_with_mark(
                node,
                marks,
                json!({"type": "link", "attrs": {"href": link.url}}),
                softbreak_as_hard,
            ),
            NodeValue::Image(link) => {
                let alt = plain_text_for_node(node).trim().to_owned();
                if alt.is_empty() {
                    vec![inline_card(&link.url, "")]
                } else {
                    self.convert_with_mark(
                        node,
                        marks,
                        json!({"type": "link", "attrs": {"href": link.url}}),
                        softbreak_as_hard,
                    )
                }
            }
            NodeValue::FootnoteReference(reference) => {
                self.text_nodes(&format!("[^{}]", reference.name), marks)
            }
            NodeValue::Math(math) => vec![adf_text(&math.literal, &[json!({"type": "code"})])],
            NodeValue::Escaped => {
                self.convert_inlines_with_softbreak(node, marks, softbreak_as_hard)
            }
            NodeValue::EscapedTag(tag) => self.text_nodes(tag, marks),
            _ => self.convert_inlines_with_softbreak(node, marks, softbreak_as_hard),
        }
    }

    fn convert_with_mark<'arena>(
        &mut self,
        node: &'arena AstNode<'arena>,
        marks: &[Value],
        mark: Value,
        softbreak_as_hard: bool,
    ) -> Vec<Value> {
        let mut child_marks = marks.to_vec();
        child_marks.push(mark);
        self.convert_inlines_with_softbreak(node, &child_marks, softbreak_as_hard)
    }

    fn text_nodes(&self, text: &str, marks: &[Value]) -> Vec<Value> {
        if text.is_empty() {
            return Vec::new();
        }

        if has_code_mark(marks) || self.options.mentions.is_empty() {
            return vec![adf_text(text, marks)];
        }

        let mut nodes = Vec::new();
        let mut pending_start = 0;
        let mut index = 0;
        while index < text.len() {
            let rest = &text[index..];
            let previous = text[..index].chars().next_back();
            if previous.is_none_or(can_start_mention_after)
                && let Some((name, consumed)) = mention_name_at_start(rest)
                && let Some(mention) = find_mention_mapping(name, &self.options.mentions)
            {
                push_text_slice(&mut nodes, &text[pending_start..index], marks);
                nodes.push(adf_mention(name, &mention.account_id));
                index += consumed;
                pending_start = index;
                continue;
            }

            let Some(ch) = rest.chars().next() else {
                break;
            };
            index += ch.len_utf8();
        }
        push_text_slice(&mut nodes, &text[pending_start..], marks);
        nodes
    }

    fn local_id<'arena>(&mut self, prefix: &str, node: &'arena AstNode<'arena>) -> String {
        let line = node.data().sourcepos.start.line;
        if line > 0 {
            format!("{prefix}-{line}")
        } else {
            self.generated_local_id += 1;
            format!("{prefix}-{}", self.generated_local_id)
        }
    }
}

fn is_table_node<'arena>(node: &'arena AstNode<'arena>) -> bool {
    matches!(node.data().value, NodeValue::Table(_))
}

fn table_numbered_rows_directive_from_node<'arena>(node: &'arena AstNode<'arena>) -> Option<bool> {
    match &node.data().value {
        NodeValue::HtmlBlock(html) => parse_table_numbered_rows_directive(&html.literal),
        _ => None,
    }
}

fn parse_table_numbered_rows_directive(line: &str) -> Option<bool> {
    let inner = line
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    let mut parts = inner.split_whitespace();
    if parts.next()? != "atla:table" {
        return None;
    }

    parts.find_map(|part| {
        if part == "numbered-rows" {
            return Some(true);
        }
        part.strip_prefix("numbered-rows=")
            .and_then(parse_directive_bool)
    })
}

fn parse_directive_bool(value: &str) -> Option<bool> {
    match value
        .trim_matches(|ch| ch == '\'' || ch == '"')
        .to_ascii_lowercase()
        .as_str()
    {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    }
}

pub fn markdown_mention_candidates(markdown: &str) -> Vec<String> {
    let arena = Arena::new();
    let comrak_options = markdown_parse_options();
    let root = parse_document(&arena, markdown, &comrak_options);
    let mut candidates = Vec::new();
    collect_mention_candidates_from_node(root, &mut candidates);
    candidates
}

fn collect_mention_candidates_from_node<'arena>(
    node: &'arena AstNode<'arena>,
    candidates: &mut Vec<String>,
) {
    match &node.data().value {
        NodeValue::Text(text) => collect_mention_candidates_from_text(text, candidates),
        NodeValue::Code(_) | NodeValue::CodeBlock(_) => {}
        _ => {
            for child in node.children() {
                collect_mention_candidates_from_node(child, candidates);
            }
        }
    }
}

fn collect_mention_candidates_from_text(text: &str, candidates: &mut Vec<String>) {
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        let previous = text[..index].chars().next_back();
        if previous.is_none_or(can_start_mention_after)
            && let Some((name, consumed)) = mention_name_at_start(rest)
        {
            push_unique_mention_candidate(candidates, name);
            index += consumed;
            continue;
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };
        index += ch.len_utf8();
    }
}

fn push_unique_mention_candidate(candidates: &mut Vec<String>, name: &str) {
    let normalized = normalize_mention_name(name);
    if normalized.is_empty() {
        return;
    }
    if !candidates
        .iter()
        .any(|candidate| normalize_mention_name(candidate) == normalized)
    {
        candidates.push(name.trim().to_owned());
    }
}

fn can_start_mention_after(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '(' | '[' | '{' | '>' | ':' | ';' | ',')
}

fn mention_name_at_start(text: &str) -> Option<(&str, usize)> {
    let rest = text.strip_prefix('@')?;
    if let Some(after_open) = rest.strip_prefix('[') {
        let end = after_open.find(']')?;
        let name = after_open[..end].trim();
        if name.is_empty() {
            return None;
        }
        return Some((name, end + 3));
    }

    let mut end = 0;
    for (offset, ch) in rest.char_indices() {
        if is_simple_mention_char(ch) {
            end = offset + ch.len_utf8();
        } else {
            break;
        }
    }
    let mut name = &rest[..end];
    while let Some(ch) = name.chars().next_back()
        && matches!(ch, '.' | ',' | ':' | ';' | '!' | '?')
    {
        let new_end = name.len() - ch.len_utf8();
        name = &name[..new_end];
    }
    if name.is_empty() {
        return None;
    }
    Some((name, 1 + name.len()))
}

fn is_simple_mention_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.')
}

fn find_mention_mapping<'a>(
    name: &str,
    mentions: &'a [MarkdownMention],
) -> Option<&'a MarkdownMention> {
    let normalized_name = normalize_mention_name(name);
    mentions
        .iter()
        .find(|mention| normalize_mention_name(&mention.text) == normalized_name)
}

fn normalize_mention_name(name: &str) -> String {
    let trimmed = name.trim();
    let without_at = trimmed.strip_prefix('@').unwrap_or(trimmed);
    let without_brackets = without_at
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(without_at);
    without_brackets.trim().to_lowercase()
}

fn adf_mention(text: &str, account_id: &str) -> Value {
    let text = text.trim().trim_start_matches('@');
    json!({
        "type": "mention",
        "attrs": {
            "id": account_id,
            "text": format!("@{text}"),
        }
    })
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

fn push_text_slice(nodes: &mut Vec<Value>, text: &str, marks: &[Value]) {
    if !text.is_empty() {
        nodes.push(adf_text(text, marks));
    }
}

fn adf_text(text: &str, marks: &[Value]) -> Value {
    if text.is_empty() {
        return json!({ "type": "text", "text": text });
    }

    let marks = canonical_adf_marks(marks);
    if marks.is_empty() {
        json!({ "type": "text", "text": text })
    } else {
        json!({ "type": "text", "text": text, "marks": marks })
    }
}

fn canonical_adf_marks(marks: &[Value]) -> Vec<Value> {
    if has_code_mark(marks) {
        return vec![json!({ "type": "code" })];
    }

    let mut marks = marks.to_vec();
    marks.sort_by_key(mark_rank);
    marks.dedup_by(|a, b| mark_key(a) == mark_key(b));
    marks
}

fn has_code_mark(marks: &[Value]) -> bool {
    marks
        .iter()
        .any(|mark| mark.get("type").and_then(Value::as_str) == Some("code"))
}

fn mark_rank(mark: &Value) -> usize {
    match mark.get("type").and_then(Value::as_str) {
        Some("em") => 0,
        Some("strong") => 1,
        Some("strike") => 2,
        Some("underline") => 3,
        Some("link") => 4,
        Some("subsup") => 5,
        Some("backgroundColor") => 6,
        _ => 100,
    }
}

fn mark_key(mark: &Value) -> String {
    match mark.get("type").and_then(Value::as_str) {
        Some("link") => format!(
            "link:{}",
            mark.get("attrs")
                .and_then(|attrs| attrs.get("href"))
                .and_then(Value::as_str)
                .unwrap_or_default()
        ),
        Some(mark_type) => mark_type.to_owned(),
        None => String::new(),
    }
}

fn panel_type_for_alert(alert_type: AlertType) -> &'static str {
    match alert_type {
        AlertType::Note | AlertType::Important => "info",
        AlertType::Tip => "success",
        AlertType::Warning => "warning",
        AlertType::Caution => "error",
    }
}

fn panel_type_for_directive(info: &str) -> &'static str {
    match info
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "note" | "info" => "info",
        "tip" | "success" => "success",
        "warning" | "warn" => "warning",
        "caution" | "danger" | "error" => "error",
        _ => "info",
    }
}

fn sanitize_blockquote_adf_blocks(blocks: Vec<Value>) -> Vec<Value> {
    blocks
        .into_iter()
        .filter_map(|block| {
            let block_type = block
                .as_object()
                .and_then(|object| object.get("type"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            if matches!(
                block_type,
                "paragraph"
                    | "orderedList"
                    | "bulletList"
                    | "codeBlock"
                    | "mediaSingle"
                    | "mediaGroup"
                    | "extension"
            ) {
                Some(block)
            } else {
                let text =
                    trim_blank_lines(&render_block(&block, 0, AdfToMarkdownOptions::default()));
                if text.is_empty() {
                    None
                } else {
                    Some(json!({ "type": "paragraph", "content": [adf_text(&text, &[])] }))
                }
            }
        })
        .collect()
}

fn plain_text_for_node<'arena>(node: &'arena AstNode<'arena>) -> String {
    let mut text = String::new();
    append_plain_text_for_node(node, &mut text);
    text
}

fn append_plain_text_for_node<'arena>(node: &'arena AstNode<'arena>, text: &mut String) {
    match &node.data().value {
        NodeValue::Text(value) => text.push_str(value),
        NodeValue::SoftBreak | NodeValue::LineBreak => text.push('\n'),
        NodeValue::Code(code) => text.push_str(&code.literal),
        NodeValue::HtmlInline(html) | NodeValue::Raw(html) => text.push_str(html),
        NodeValue::CodeBlock(code) => text.push_str(&code.literal),
        NodeValue::HtmlBlock(html) => text.push_str(&html.literal),
        NodeValue::FootnoteReference(reference) => {
            text.push_str(&format!("[^{}]", reference.name));
        }
        NodeValue::FootnoteDefinition(footnote) => {
            text.push_str(&format!("[^{}]: ", footnote.name));
            append_plain_text_for_children(node, text);
        }
        NodeValue::Math(math) => text.push_str(&math.literal),
        NodeValue::ThematicBreak => text.push_str("---"),
        NodeValue::Image(link) => {
            let before_image = text.len();
            append_plain_text_for_children(node, text);
            if text.len() == before_image {
                text.push_str(&link.url);
            }
        }
        _ => append_plain_text_for_children(node, text),
    }
}

fn append_plain_text_for_children<'arena>(node: &'arena AstNode<'arena>, text: &mut String) {
    let mut first = true;
    for child in node.children() {
        if !first && child.data().value.block() && !text.ends_with('\n') {
            text.push('\n');
        }
        append_plain_text_for_node(child, text);
        first = false;
    }
}
