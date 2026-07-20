mod parse;
mod render;

pub use parse::markdown_mention_candidates;
use parse::parse_markdown_blocks;
use render::{render_block, trim_blank_lines};
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

pub fn adf_to_markdown(adf: &Value) -> String {
    adf_to_markdown_with_options(adf, AdfToMarkdownOptions::default())
}

pub fn adf_to_markdown_with_options(adf: &Value, options: AdfToMarkdownOptions) -> String {
    trim_blank_lines(&render_block(adf, 0, options))
}

#[cfg(test)]
mod tests;
