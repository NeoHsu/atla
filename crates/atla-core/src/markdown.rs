use serde_json::{Value, json};

pub fn markdown_to_adf(markdown: &str) -> Value {
    json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": markdown
                    }
                ]
            }
        ]
    })
}

pub fn adf_to_markdown(adf: &Value) -> String {
    let mut out = String::new();
    render_node(adf, &mut out);
    out.trim().to_owned()
}

fn render_node(value: &Value, out: &mut String) {
    match value {
        Value::Array(items) => {
            for item in items {
                render_node(item, out);
            }
        }
        Value::Object(object) => {
            let node_type = object
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            match node_type {
                "paragraph" => {
                    render_node_value(object.get("content"), out);
                    out.push_str("\n\n");
                }
                "heading" => {
                    let level = object
                        .get("attrs")
                        .and_then(|attrs| attrs.get("level"))
                        .and_then(Value::as_u64)
                        .unwrap_or(1)
                        .clamp(1, 6);
                    out.push_str(&"#".repeat(level as usize));
                    out.push(' ');
                    render_node_value(object.get("content"), out);
                    out.push_str("\n\n");
                }
                "bulletList" => render_list(object.get("content"), out, "- "),
                "orderedList" => render_list(object.get("content"), out, "1. "),
                "listItem" => {
                    render_node_value(object.get("content"), out);
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
                "text" => {
                    if let Some(text) = object.get("text").and_then(Value::as_str) {
                        out.push_str(text);
                    }
                }
                "hardBreak" => out.push('\n'),
                "codeBlock" => {
                    out.push_str("```\n");
                    render_node_value(object.get("content"), out);
                    out.push_str("\n```\n\n");
                }
                _ => render_node_value(object.get("content"), out),
            }
        }
        _ => {}
    }
}

fn render_node_value(value: Option<&Value>, out: &mut String) {
    if let Some(value) = value {
        render_node(value, out);
    }
}

fn render_list(value: Option<&Value>, out: &mut String, marker: &str) {
    let Some(Value::Array(items)) = value else {
        return;
    };
    for item in items {
        out.push_str(marker);
        let before = out.len();
        render_node(item, out);
        let item_text = out[before..].trim().to_owned();
        out.truncate(before);
        out.push_str(&item_text);
        out.push('\n');
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
