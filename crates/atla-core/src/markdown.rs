use comrak::nodes::{AlertType, AstNode, ListType, NodeValue};
use comrak::{Arena, Options, parse_document};
use serde_json::{Value, json};

#[derive(Debug, Clone, Default)]
pub struct MarkdownToAdfOptions {
    pub numbered_table_rows: bool,
    pub mentions: Vec<MarkdownMention>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownMention {
    pub text: String,
    pub account_id: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AdfToMarkdownOptions {
    pub table_numbered_rows_directives: bool,
}

pub fn markdown_to_adf(markdown: &str) -> Value {
    markdown_to_adf_with_options(markdown, MarkdownToAdfOptions::default())
}

pub fn markdown_to_adf_with_options(markdown: &str, options: MarkdownToAdfOptions) -> Value {
    let blocks = parse_markdown_blocks(markdown, &options);
    json!({
        "type": "doc",
        "version": 1,
        "content": blocks,
    })
}

fn parse_markdown_blocks(markdown: &str, options: &MarkdownToAdfOptions) -> Vec<Value> {
    let arena = Arena::new();
    let comrak_options = markdown_parse_options();
    let root = parse_document(&arena, markdown, &comrak_options);
    MarkdownAdfConverter::new(options).convert_document(root)
}

pub fn adf_to_markdown(adf: &Value) -> String {
    adf_to_markdown_with_options(adf, AdfToMarkdownOptions::default())
}

pub fn adf_to_markdown_with_options(adf: &Value, options: AdfToMarkdownOptions) -> String {
    trim_blank_lines(&render_block(adf, 0, options))
}

fn render_block(value: &Value, depth: usize, options: AdfToMarkdownOptions) -> String {
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
}
