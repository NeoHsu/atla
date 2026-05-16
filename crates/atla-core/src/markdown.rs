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
    adf.to_string()
}
