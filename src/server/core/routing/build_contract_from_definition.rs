use crate::server::contracts::users;
use crate::server::core::manifest::manifest::ContractDefinition;
use crate::server::core::manifest::manifest_wrapper::ManifestContractWrapper;
use crate::server::core::routing::DynamicContract;
use std::sync::Arc;

pub fn build_contract_from_definition(
    def: &ContractDefinition,
) -> Result<Arc<dyn DynamicContract>, Box<dyn std::error::Error>> {
    match def.id.as_str() {
        "users::get_by_id" => {
            println!("Compiling pipeline node: {}", def.id);

            let pure_contract = users::get_by_id::DynamicGetById;

            let runner = ManifestContractWrapper {
                inner_contract: pure_contract,
                input_mappings: def.inputs.clone(),
                target_store_key: def.target_store_key.clone(),
            };

            Ok(Arc::new(runner))
        }

         "users::create" => {
            println!("Compiling pipeline node: {}", def.id);

            let pure_contract = users::create::Create;

            let runner = ManifestContractWrapper {
                inner_contract: pure_contract,
                input_mappings: def.inputs.clone(),
                target_store_key: None,
            };

            Ok(Arc::new(runner))
        }

        unsupported => Err(format!(
            "Initialization Error: Spec definition module '{}' is not registered in this runtime host environment.",
            unsupported
        )
        .into()),
    }
}
