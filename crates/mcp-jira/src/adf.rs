use serde_json::{json, Value};

/// Convert plain text to Atlassian Document Format (ADF).
pub fn text_to_adf(text: &str) -> Value {
    let paragraphs: Vec<Value> = text
        .split('\n')
        .map(|line| {
            if line.is_empty() {
                json!({
                    "type": "paragraph",
                    "content": []
                })
            } else {
                json!({
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": line
                        }
                    ]
                })
            }
        })
        .collect();

    json!({
        "version": 1,
        "type": "doc",
        "content": paragraphs
    })
}

/// Extract plain text from an ADF document (best-effort).
pub fn adf_to_text(adf: &Value) -> String {
    let mut result = String::new();
    extract_text_recursive(adf, &mut result);
    result.trim_end().to_string()
}

fn extract_text_recursive(node: &Value, out: &mut String) {
    if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
        out.push_str(text);
    }
    if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
        for child in content {
            extract_text_recursive(child, out);
        }
        // Add newline after block-level elements
        if let Some(
            "paragraph" | "heading" | "bulletList" | "orderedList" | "listItem"
            | "blockquote" | "codeBlock" | "table" | "tableRow",
        ) = node.get("type").and_then(|t| t.as_str())
        {
            out.push('\n');
        }
    }
}
