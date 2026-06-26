// In src/server/core/routing/build_contract_from_definition.rs

use crate::server::core::manifest::manifest::ContractDefinition;
//use crate::server::core::manifest::manifest_wrapper::ManifestContractWrapper;
use crate::server::core::routing::DynamicContract;
use crate::server::functions::wasm_functions::WasmContractRunner; // Your WASM step wrapper
use crate::server::functions::wasm_functions::WasmRuntime;
use std::collections::HashMap;
use std::sync::Arc;
use wasmtime::Module;

pub fn build_contract_from_definition(
    def: &ContractDefinition,
    runtime: Arc<WasmRuntime>,
    compiled_contracts: &HashMap<String, Module>, // 💡 Pass the pre-compiled WASM cache
) -> Result<Arc<dyn DynamicContract>, Box<dyn std::error::Error>> {
    // 1. 💡 CHECK: If the contract ID exists in our pre-compiled WASM cache, build a WASM step!
    if let Some(compiled_module) = compiled_contracts.get(&def.id) {
        let wasm_runner = WasmContractRunner {
            runtime,
            module: compiled_module.clone(),
            input_mappings: def.inputs.clone(),
            set_store_mappings: def.set_store_mappings.clone(),
        };
        return Ok(Arc::new(wasm_runner));
    }

    // 2. FALLBACK: If it's not a WASM module, match against your hardcoded native Rust implementations
    match def.id.as_str() {
        /*"users::get_by_id" => {
            println!("Compiling native pipeline node: {}", def.id);
            let pure_contract = users::get_by_id::GetById;

            let runner = ManifestContractWrapper {
                inner_contract: pure_contract,
                input_mappings: def.inputs.clone(),
                set_store_mappings: def.set_store_mappings.clone(),
            };
            Ok(Arc::new(runner))
        }

        "users::create" => {
            println!("Compiling native pipeline node: {}", def.id);
            let pure_contract = users::create::Create;

            let runner = ManifestContractWrapper {
                inner_contract: pure_contract,
                input_mappings: def.inputs.clone(),
                set_store_mappings: def.set_store_mappings.clone(),
            };
            Ok(Arc::new(runner))
        }*/

        unsupported => Err(format!(
            "Initialization Error: Spec definition module '{}' is not registered in this runtime host environment.",
            unsupported
        )
        .into()),
    }
}
