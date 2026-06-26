// In src/main.rs

pub mod server;

use actix_web::{App, HttpServer, web};
use std::env;
use std::sync::Arc;

use crate::server::core::routing::DynamicContract;
use crate::server::core::routing::build_contract_from_definition::build_contract_from_definition;
use crate::server::core::routing::dynamic_pipeline::DynamicPipeline;
use crate::server::core::routing::dynamic_route::dynamic_route;
use crate::server::core::workspace::AppWorkspace;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Grab the workspace folder from the CLI arguments (defaulting to ./server1)
    let args: Vec<String> = env::args().collect();
    let target_workspace = if args.len() >= 3 && args[1] == "serve" {
        &args[2]
    } else {
        "./server1"
    };

    // 2. Bootstrap the entire workspace context dynamically from disk filesystem hierarchy
    let workspace_state = match AppWorkspace::bootstrap(target_workspace) {
        Ok(ws) => Arc::new(ws),
        Err(err) => {
            eprintln!(
                "CRITICAL: Failed to bootstrap filesystem workspace at '{}': {}",
                target_workspace, err
            );
            std::process::exit(1);
        }
    };

    println!("Starting Dynamic Specification Server on 127.0.0.1:8080...");

    // 3. Boot up the server thread factory, building pipelines from the crawled state
    HttpServer::new(move || {
        let mut app = App::new();

        // Loop directly through the dynamic filesystem routes discovered by our workspace bootstrapper
        for discovered_route in &workspace_state.routes {
            let mut compiled_steps: Vec<Arc<dyn DynamicContract>> = Vec::new();
            let route_def = &discovered_route.definition;

            // Assemble the pipeline execution steps dynamically for each route config
            for contract_def in &route_def.contracts {
                match build_contract_from_definition(
                    contract_def,
                    Arc::clone(&workspace_state.runtime),
                    &workspace_state.compiled_contracts,
                ) {
                    Ok(contract_arc) => compiled_steps.push(contract_arc),
                    Err(err) => {
                        eprintln!(
                            "Compilation Error inside route factory '{}': {}",
                            discovered_route.path, err
                        );
                        std::process::exit(1);
                    }
                }
            }

            let route_path = discovered_route.path.clone();
            let pipeline_state = web::Data::new(DynamicPipeline {
                steps: compiled_steps,
            });
            let output_mapping_state = web::Data::new(route_def.output.clone());

            // Match the specific HTTP verb extracted during filesystem crawling
            let route_guard = match discovered_route.method.as_str() {
                "POST" => web::post(),
                "PUT" => web::put(),
                "DELETE" => web::delete(),
                "PATCH" => web::patch(),
                _ => web::get(),
            };

            // Bind the states directly underneath the scoped path mapping
            app = app.service(
                web::scope(&route_path)
                    .app_data(pipeline_state)
                    .app_data(output_mapping_state)
                    .route("", route_guard.to(dynamic_route)),
            );
        }

        app
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
