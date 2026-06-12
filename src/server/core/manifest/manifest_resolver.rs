use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Evaluates whether a manifest rule value is a dynamic context pointer or a static value literal
// 💡 FIXED: We pass the explicit thread-safe components instead of the whole non-Send DynamicContext
pub async fn resolve_manifest_value(
    rule_value: &Value,
    request_data: &Value,
    store: &Arc<RwLock<serde_json::Map<String, Value>>>,
) -> Value {
    if let Some(str_val) = rule_value.as_str() {
        if str_val.starts_with("ctx(") && str_val.ends_with(')') {
            let inner_path = &str_val[4..str_val.len() - 1];

            let extracted_value = if inner_path.starts_with("/store/") {
                // 💡 SAFE: RwLockGuard across an await point is thread-safe here
                let store_guard = store.read().await;
                let store_path = inner_path.replacen("/store", "", 1);

                let temp_wrapped_store = Value::Object(store_guard.clone());
                temp_wrapped_store.pointer(&store_path).cloned()
            } else {
                request_data.pointer(inner_path).cloned()
            };

            return extracted_value.unwrap_or(Value::Null);
        }
    }

    rule_value.clone()
}
