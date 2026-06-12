use crate::server::core::manifest::manifest_resolver::resolve_manifest_value;
use crate::server::core::routing::dynamic_context::DynamicContext;
use crate::server::core::routing::{DynamicContract, InnerContract};
use futures::future::{BoxFuture, FutureExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub trait IntoJsonValue {
    fn into_json(self) -> Value;
}

impl IntoJsonValue for Value {
    fn into_json(self) -> Value {
        self
    }
}

impl IntoJsonValue for () {
    fn into_json(self) -> Value {
        Value::Null
    }
}

pub struct ManifestContractWrapper<T> {
    pub inner_contract: T,
    pub input_mappings: HashMap<String, Value>,
    // 💡 CHANGED: Changed type to Option<String>
    pub target_store_key: Option<String>,
}

impl<T> DynamicContract for ManifestContractWrapper<T>
where
    T: InnerContract + 'static,
    T::Output: IntoJsonValue,
{
    fn execute(self: Arc<Self>, ctx: DynamicContext) -> BoxFuture<'static, Result<(), String>> {
        let mappings = self.input_mappings.clone();
        let target_key = self.target_store_key.clone();

        async move {
            let mut constructed_input_json = serde_json::Map::new();
            let request_data = &ctx.request;
            let pipeline_store = &ctx.store;

            for (field_name, rule_value) in mappings {
                let val = resolve_manifest_value(&rule_value, request_data, pipeline_store).await;
                constructed_input_json.insert(field_name, val);
            }

            let typed_input =
                serde_json::from_value::<T::Input>(Value::Object(constructed_input_json))
                    .map_err(|e| format!("Input argument structural mismatch: {}", e))?;

            self.inner_contract.validate_pure(&typed_input)?;
            let raw_output = self.inner_contract.commit_pure(typed_input).await?;

            // 💡 FIXED: Only lock and write to the pipeline store if a target key was configured!
            if let Some(key) = target_key {
                let output_value: Value = raw_output.into_json();
                let mut store_guard = ctx.store.write().await;
                store_guard.insert(key, output_value);
            }

            Ok(())
        }
        .boxed()
    }
}
