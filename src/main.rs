pub mod server;

use actix_web::{App, HttpServer, web};
use std::sync::Arc;

use crate::server::core::manifest::manifest::load_manifest_from_file;
use crate::server::core::routing::DynamicContract;
use crate::server::core::routing::build_contract_from_definition::build_contract_from_definition;
use crate::server::core::routing::dynamic_pipeline::DynamicPipeline;
use crate::server::core::routing::dynamic_route::dynamic_route;
use std::collections::HashMap;

struct RuntimeRouteBlueprint {
    path: String,
    pipeline: DynamicPipeline,
    output_mapping: HashMap<String, serde_json::Value>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Read the JSON server specification from disk
    let manifest_path = "./data/server1.json";
    let manifest = match load_manifest_from_file(manifest_path) {
        Ok(m) => m,
        Err(err) => {
            eprintln!(
                "CRITICAL: Failed to load manifest file '{}': {}",
                manifest_path, err
            );
            std::process::exit(1);
        }
    };

    println!(
        "Loading workspace: {} (v{})",
        manifest.server_name, manifest.version
    );

    // 2. Translate the structural definitions into live execution pipelines
    let mut runtime_blueprints = Vec::new();

    for route_def in manifest.routes {
        let mut compiled_steps: Vec<Arc<dyn DynamicContract>> = Vec::new();

        for contract_def in route_def.contracts {
            match build_contract_from_definition(&contract_def) {
                Ok(contract_arc) => compiled_steps.push(contract_arc),
                Err(err) => {
                    eprintln!("Compilation Error in route '{}': {}", route_def.path, err);
                    std::process::exit(1);
                }
            }
        }

        runtime_blueprints.push(RuntimeRouteBlueprint {
            path: route_def.path,
            pipeline: DynamicPipeline {
                steps: compiled_steps,
            },
            output_mapping: route_def.output,
        });
    }

    let shared_blueprints = Arc::new(runtime_blueprints);

    println!("Starting Dynamic Specification Server on 127.0.0.1:8080...");

    // 3. Boot up the server thread factory
    HttpServer::new(move || {
        let mut app = App::new();

        for blueprint in shared_blueprints.iter() {
            let route_path = blueprint.path.clone();
            let pipeline_state = web::Data::new(blueprint.pipeline.clone());
            let output_mapping_state = web::Data::new(blueprint.output_mapping.clone());

            app = app.service(
                web::scope(&route_path)
                    .app_data(pipeline_state)
                    .app_data(output_mapping_state)
                    .route("", web::get().to(dynamic_route)),
            );
        }

        app
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
