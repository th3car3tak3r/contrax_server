// In src/server/core/routing/build_contract_from_definition.rs

use crate::server::contracts::web::fetch::FetchContract;
use crate::server::core::manifest::manifest::ContractDefinition;
use crate::server::core::manifest::manifest_wrapper::ManifestContractWrapper;
use crate::server::core::routing::DynamicContract;
use crate::server::functions::wasm_functions::WasmContractRunner;
use crate::server::functions::wasm_functions::WasmRuntime;
use std::collections::HashMap;
use std::sync::Arc;
use wasmtime::Module;

pub fn build_contract_from_definition(
    def: &ContractDefinition,
    runtime: Arc<WasmRuntime>,
    compiled_contracts: &HashMap<String, Module>,
) -> Result<Arc<dyn DynamicContract>, Box<dyn std::error::Error>> {
    // 1. CHECK: If the contract ID exists in our pre-compiled WASM cache, build a WASM step!
    if let Some(compiled_module) = compiled_contracts.get(&def.id) {
        let wasm_runner = WasmContractRunner {
            runtime,
            module: compiled_module.clone(),
            input_mappings: def.inputs.clone(),
            set_store_mappings: def.set_store_mappings.clone(),
        };
        return Ok(Arc::new(wasm_runner));
    }

    // 2. FALLBACK: Match against your hardcoded native Rust contract implementations
   // Inside src/server/core/routing/build_contract_from_definition.rs

    match def.id.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" => {
            println!("Compiling native HTTP pipeline gateway: {}", def.id);
            // 💡 Pass the id string ("GET", "POST", etc.) right into the constructor!
            let pure_contract = FetchContract::new(def.id.as_str());
            
            let runner = ManifestContractWrapper {
                inner_contract: pure_contract,
                input_mappings: def.inputs.clone(),
                set_store_mappings: def.set_store_mappings.clone(),
            };
            Ok(Arc::new(runner))
        }

        /*"users::get_by_id" => { ... }*/

        unsupported => Err(format!(
            "Initialization Error: Spec definition module '{}' is not registered in this runtime host environment.",
            unsupported
        )
        .into()),
    }

    
}
