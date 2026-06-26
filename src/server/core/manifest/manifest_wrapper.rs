use crate::server::core::manifest::manifest_resolver::resolve_manifest_value;
use crate::server::core::routing::dynamic_context::DynamicContext;
use crate::server::core::routing::{Contract, DynamicContract};
use futures::future::{BoxFuture, FutureExt};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ManifestContractWrapper<T> {
    pub inner_contract: T,
    pub input_mappings: HashMap<String, Value>,
    pub set_store_mappings: Option<serde_json::Map<String, Value>>,
}

fn resolve_nested_store_value(manifest_value: &Value, out_json: &Value) -> Result<Value, String> {
    match manifest_value {
        Value::Object(map) => {
            let mut resolved_map = serde_json::Map::new();
            for (key, val) in map {
                let resolved_val = resolve_nested_store_value(val, out_json)?;
                resolved_map.insert(key.clone(), resolved_val);
            }
            Ok(Value::Object(resolved_map))
        }
        Value::Array(arr) => {
            let mut resolved_arr = Vec::new();
            for val in arr {
                let resolved_val = resolve_nested_store_value(val, out_json)?;
                resolved_arr.push(resolved_val);
            }
            Ok(Value::Array(resolved_arr))
        }
        Value::String(pointer_str) => {
            if pointer_str.starts_with("out(") && pointer_str.ends_with(')') {
                let extracted_inner = pointer_str
                    .strip_prefix("out(")
                    .and_then(|s| s.strip_suffix(')'))
                    .ok_or_else(|| format!("Malformed pointer sequence string: {}", pointer_str))?;

                let json_pointer = if extracted_inner.starts_with('/') {
                    extracted_inner.to_string()
                } else {
                    format!("/{}", extracted_inner)
                };

                if let Some(extracted_value) = out_json.pointer(&json_pointer) {
                    Ok(extracted_value.clone())
                } else {
                    Err(format!(
                        "Pointer resolution fault: path '{}' not found within contract output schema.",
                        json_pointer
                    ))
                }
            } else {
                Ok(Value::String(pointer_str.clone()))
            }
        }
        fallback => Ok(fallback.clone()),
    }
}

impl<T> DynamicContract for ManifestContractWrapper<T>
where
    T: Contract + 'static,
    T::Output: Serialize + Send + 'static,
{
    fn execute(self: Arc<Self>, ctx: DynamicContext) -> BoxFuture<'static, Result<(), String>> {
        let inputs_blueprint = self.input_mappings.clone();
        let store_blueprint = self.set_store_mappings.clone();

        async move {
            let mut constructed_input_json = serde_json::Map::new();
            let request_data = &ctx.request;
            let pipeline_store = &ctx.store;

            // 1. Core Data Aggregation & Value Resolution
            for (field_name, rule_value) in inputs_blueprint {
                let val = resolve_manifest_value(&rule_value, request_data, pipeline_store).await;
                constructed_input_json.insert(field_name, val);
            }

            // 2. Strongly-Typed Transformation Layer
            let typed_input =
                serde_json::from_value::<T::Input>(Value::Object(constructed_input_json))
                    .map_err(|e| format!("Input argument structural mismatch: {}", e))?;

            // 3. Isolated Logic Ingress Boundary Execution
            self.inner_contract.validate(&typed_input)?;
            let raw_output = self.inner_contract.commit(typed_input).await?;

            // 4. Granular Store Destructuring and Mapping
            if let Some(mappings) = store_blueprint {
                let out_json = serde_json::to_value(raw_output)
                    .map_err(|e| format!("Serialization failure of contract output: {}", e))?;

                // Acquire write lock to update the global pipeline memory pool safely
                let mut store_guard = ctx.store.write().await;

                for (store_key, manifest_value) in mappings {
                    // Run our recursive builder to resolve the target value structural tree
                    let resolved_value = resolve_nested_store_value(&manifest_value, &out_json)?;

                    // Insert the fully compiled object straight into the store root key
                    store_guard.insert(store_key, resolved_value);
                }
            }

            Ok(())
        }
        .boxed()
    }
}
