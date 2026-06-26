use std::ffi::CStr;
use std::sync::Arc;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

// Imports to hook cleanly into your dynamic routing pipeline mechanics
use crate::server::core::manifest::manifest_resolver::resolve_manifest_value;
use crate::server::core::routing::DynamicContract;
use crate::server::core::routing::dynamic_context::DynamicContext;
use serde_json::Value;
use std::collections::HashMap;

// Added for your explicit trait future compliance
use futures::future::BoxFuture;

/// Shared runtime environment containing the global Wasmtime engine state.
pub struct WasmRuntime {
    pub engine: Engine,
    pub linker: Linker<wasmtime_wasi::p1::WasiP1Ctx>,
}

impl WasmRuntime {
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        config.wasm_gc(true);
        config.wasm_threads(true);

        let engine = Engine::new(&config)
            .expect("Failed to initialize system core WebAssembly engine runtime context.");

        let mut linker = Linker::<wasmtime_wasi::p1::WasiP1Ctx>::new(&engine);
        wasmtime_wasi::p1::add_to_linker_async(&mut linker, |s| s)
            .expect("Failed to bind async WASI P1 capabilities to host linker instance.");

        Self { engine, linker }
    }
}

/// PIPELINE BRIDGE: Implements your explicit DynamicContract box-future layout.
pub struct WasmContractRunner {
    pub runtime: Arc<WasmRuntime>,
    pub module: Module,
    pub input_mappings: HashMap<String, Value>,
    pub set_store_mappings: Option<serde_json::Map<String, Value>>,
}

impl DynamicContract for WasmContractRunner {
    fn execute(self: Arc<Self>, ctx: DynamicContext) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async move {
            let request_data = &ctx.request;
            let pipeline_store = &ctx.store;

            // 1. Resolve inputs out of the context parameters dynamically using your resolver
            let mut contract_input_payload = serde_json::Map::new();
            for (input_key, rule_value) in &self.input_mappings {
                let final_val =
                    resolve_manifest_value(rule_value, request_data, pipeline_store).await;
                contract_input_payload.insert(input_key.clone(), final_val);
            }

            let execution_context_string = Value::Object(contract_input_payload).to_string();

            // 2. Invoke the sandboxed runtime boundary execution layout
            let raw_wasm_output = run_wasm_contract(
                Arc::clone(&self.runtime),
                &self.module,
                &execution_context_string,
            )
            .await;

            // 3. Parse output payload back into a JSON object representation
            let wasm_response_json: Value =
                serde_json::from_str(&raw_wasm_output).map_err(|e| {
                    format!(
                        "WASM Pipeline output extraction failed serialization: {}",
                        e
                    )
                })?;

            // 4. Populate step memory space context store matching your declarative extraction definitions
            if let Some(store_rules) = &self.set_store_mappings {
                let mut store_lock = ctx.store.write().await;

                for (store_key, pattern_rule) in store_rules {
                    match pattern_rule {
                        // Handle structured nested dictionary targets
                        Value::Object(pattern_map) => {
                            let mut resolved_sub_object = serde_json::Map::new();

                            for (field_name, extraction_expression) in pattern_map {
                                if let Value::String(expr_str) = extraction_expression {
                                    if expr_str.starts_with("out(") && expr_str.ends_with(')') {
                                        let property_path = &expr_str[4..expr_str.len() - 1];
                                        if let Some(extracted_val) =
                                            wasm_response_json.get(property_path)
                                        {
                                            resolved_sub_object
                                                .insert(field_name.clone(), extracted_val.clone());
                                        } else {
                                            resolved_sub_object
                                                .insert(field_name.clone(), Value::Null);
                                        }
                                    } else {
                                        resolved_sub_object.insert(
                                            field_name.clone(),
                                            Value::String(expr_str.clone()),
                                        );
                                    }
                                } else {
                                    resolved_sub_object
                                        .insert(field_name.clone(), extraction_expression.clone());
                                }
                            }
                            store_lock
                                .insert(store_key.clone(), Value::Object(resolved_sub_object));
                        }
                        // 💡 FIX: Handle straight standalone mapping rules (e.g., "test_success": "out(success)")
                        Value::String(expr_str) => {
                            if expr_str.starts_with("out(") && expr_str.ends_with(')') {
                                let property_path = &expr_str[4..expr_str.len() - 1];
                                if let Some(extracted_val) = wasm_response_json.get(property_path) {
                                    store_lock.insert(store_key.clone(), extracted_val.clone());
                                } else {
                                    store_lock.insert(store_key.clone(), Value::Null);
                                }
                            } else {
                                store_lock
                                    .insert(store_key.clone(), Value::String(expr_str.clone()));
                            }
                        }
                        // Fallback for direct primitives passed down straight
                        _ => {
                            store_lock.insert(store_key.clone(), pattern_rule.clone());
                        }
                    }
                }
            }

            Ok(())
        })
    }
}

/// Executes an isolated WebAssembly contract by borrowing from the global runtime environment.
pub async fn run_wasm_contract(
    runtime: Arc<WasmRuntime>,
    module: &Module,
    request_context: &str,
) -> String {
    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .build_p1();

    let mut store = Store::new(&runtime.engine, wasi_ctx);

    let instance = runtime
        .linker
        .instantiate_async(&mut store, module)
        .await
        .expect("Sandboxed initialization error: Container failure occurred.");

    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("Failed to locate WebAssembly linear memory allocation export.");

    let alloc_fn = instance
        .get_typed_func::<u32, u32>(&mut store, "alloc")
        .expect("Failed to bind WASM memory allocator routine.");

    let execute = instance
        .get_typed_func::<u32, u32>(&mut store, "execute_contract")
        .expect("Failed to bind WASM execution contract handler.");

    let input_bytes = request_context.as_bytes();
    let input_len = input_bytes.len() as u32;

    let wasm_buffer_offset = alloc_fn
        .call_async(&mut store, input_len + 1)
        .await
        .unwrap();

    memory
        .write(&mut store, wasm_buffer_offset as usize, input_bytes)
        .expect("Linear memory ingress violation: Write overflow exception occurred.");

    memory
        .write(&mut store, (wasm_buffer_offset + input_len) as usize, &[0])
        .unwrap();

    let result_buffer_offset = execute
        .call_async(&mut store, wasm_buffer_offset)
        .await
        .unwrap();

    let linear_memory_data = memory.data(&store);

    let raw_c_string_pointer = unsafe {
        linear_memory_data
            .as_ptr()
            .add(result_buffer_offset as usize) as *const i8
    };

    let output_c_str = unsafe { CStr::from_ptr(raw_c_string_pointer) };
    let output_string = output_c_str
        .to_str()
        .expect("WASM output failed string validation: Invalid UTF-8 sequence encountered.")
        .to_string();

    output_string
}
