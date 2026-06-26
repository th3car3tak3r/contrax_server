// In src/server/core/manifest/manifest_resolver.rs

use regex::Regex;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Evaluates whether a manifest rule value is a dynamic context pointer or a static value literal
pub async fn resolve_manifest_value(
    rule_value: &Value,
    request_data: &Value,
    store: &Arc<RwLock<serde_json::Map<String, Value>>>,
) -> Value {
    if let Some(str_val) = rule_value.as_str() {
        if str_val.starts_with("ctx(") && str_val.ends_with(')') {
            let extracted_inner = &str_val[4..str_val.len() - 1];

            // Normalize the string to guarantee an RFC 6901 compliant leading slash
            let inner_path = if extracted_inner.starts_with('/') {
                extracted_inner.to_string()
            } else {
                format!("/{}", extracted_inner)
            };

            let extracted_value = if inner_path.starts_with("/store/") {
                let store_guard = store.read().await;
                let store_path = inner_path.replacen("/store", "", 1);

                let temp_wrapped_store = Value::Object(store_guard.clone());
                temp_wrapped_store.pointer(&store_path).cloned()
            } else {
                request_data.pointer(&inner_path).cloned()
            };

            return extracted_value.unwrap_or(Value::Null);
        }
    }

    rule_value.clone()
}

/// Scans a text template string for matching `{{ ctx(path/to/value) }}` contract delimiters,
/// extracts the internal pointer targets, resolves them, and returns an interpolated string.
pub async fn interpolate_string_tokens(
    template: &str,
    request_data: &Value,
    store: &Arc<RwLock<serde_json::Map<String, Value>>>,
) -> String {
    // Matches: {{ ctx(path/to/target) }} while handling loose spacing elegantly
    let re = Regex::new(r"\{\{\s*ctx\(([^)]+)\)\s*\}\}").unwrap();
    let mut result = template.to_string();

    // Collect all matching tokens requiring lookup to prevent mutation ownership collision loops
    let captures: Vec<(String, String)> = re
        .captures_iter(template)
        .filter_map(|cap| {
            let full_match = cap.get(0)?.as_str().to_string();
            let context_path = cap.get(1)?.as_str().to_string();
            Some((full_match, context_path))
        })
        .collect();

    for (full_match, context_path) in captures {
        // Construct a standard context invocation token to leverage our existing resolution worker
        let standard_token = Value::String(format!("ctx({})", context_path));
        let resolved_json_val = resolve_manifest_value(&standard_token, request_data, store).await;

        // Extract a clean string representation for standard injection targets
        let replacement_text = match resolved_json_val {
            Value::String(s) => s,
            Value::Null => "".to_string(),
            other => other.to_string(),
        };

        result = result.replace(&full_match, &replacement_text);
    }

    result
}
