// In src/server/core/workspace.rs

use crate::server::core::manifest::manifest::RouteDefinition;
use crate::server::functions::wasm_functions::WasmRuntime;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wasmtime::Module;

/// Wraps a route definition with its discovered HTTP verb and URL path pattern.
#[derive(Clone)]
pub struct DiscoveredRoute {
    pub method: String,
    pub path: String,
    pub definition: RouteDefinition,
}

#[derive(Clone)]
pub struct AppWorkspace {
    pub runtime: Arc<WasmRuntime>,
    pub compiled_contracts: HashMap<String, Module>,
    pub routes: Vec<DiscoveredRoute>,
}

impl AppWorkspace {
    pub fn bootstrap(workspace_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = Arc::new(WasmRuntime::new());
        let mut compiled_contracts = HashMap::new();
        let mut routes = Vec::new();

        let base_path = Path::new(workspace_path);

        println!("Bootstrapping file-system workspace at: {}", workspace_path);

        // 1. Discover and pre-compile all local WASM contract files
        let contracts_dir = base_path.join("contracts");
        if contracts_dir.exists() && contracts_dir.is_dir() {
            for entry in fs::read_dir(contracts_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().map_or(false, |ext| ext == "wasm") {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        println!("Pre-compiling WebAssembly contract: {}.wasm", file_stem);
                        let module = Module::from_file(&runtime.engine, &path)?;
                        compiled_contracts.insert(file_stem.to_string(), module);
                    }
                }
            }
        }

        // 2. Recursively crawl the routes directory to build the API topology
        let routes_dir = base_path.join("routes");
        if routes_dir.exists() && routes_dir.is_dir() {
            Self::crawl_routes_directory(&routes_dir, &routes_dir, &mut routes)?;
        } else {
            println!("Warning: No 'routes' directory found in workspace.");
        }

        Ok(Self {
            runtime,
            compiled_contracts,
            routes,
        })
    }

    /// Recursively walks the route file structure to map directory slugs and JSON files to endpoints.
    fn crawl_routes_directory(
        base_routes_path: &Path,
        current_dir: &Path,
        discovered: &mut Vec<DiscoveredRoute>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively traverse child directories
                Self::crawl_routes_directory(base_routes_path, &path, discovered)?;
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let method = file_stem.to_uppercase();

                    // Only process standard HTTP method configuration targets
                    if matches!(method.as_str(), "GET" | "POST" | "PUT" | "DELETE" | "PATCH") {
                        let route_path = Self::determine_route_path(base_routes_path, &path)?;

                        let config_raw = fs::read_to_string(&path)?;
                        let mut definition: RouteDefinition = serde_json::from_str(&config_raw)?;

                        // Inject the context discovered from the filesystem directly into the definition struct
                        definition.path = route_path.clone();
                        definition.method = method.clone();

                        discovered.push(DiscoveredRoute {
                            method,
                            path: route_path,
                            definition,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Transforms an absolute target file path into an Actix web-compatible path string.
    /// Example: /workspace/routes/users/{value}/get.json -> /users/{value}
    fn determine_route_path(
        base_routes_path: &Path,
        file_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let parent_dir = file_path.parent().unwrap_or(base_routes_path);

        // Strip out the base routes root path to determine the relative URL slug
        let relative = parent_dir.strip_prefix(base_routes_path)?;
        let relative_str = relative.to_string_lossy().into_owned();

        if relative_str.is_empty() {
            Ok("/".to_string())
        } else {
            // Ensure path starts with a clean single forward slash and standard separators
            let normalized = relative_str.replace('\\', "/");
            Ok(format!("/{}", normalized))
        }
    }
}
