use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct ContractDefinition {
    pub id: String,
    pub inputs: HashMap<String, Value>,
    // 💡 CHANGED: Made optional so it can be omitted in the JSON manifest
    pub target_store_key: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RouteDefinition {
    pub name: String,
    pub path: String,
    pub contracts: Vec<ContractDefinition>,
    pub output: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerManifest {
    pub server_name: String,
    pub version: String,
    pub routes: Vec<RouteDefinition>,
}

/// Reads a server manifest specification JSON file from disk and parses it
pub fn load_manifest_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<ServerManifest, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut json_content = String::new();
    file.read_to_string(&mut json_content)?;

    let manifest: ServerManifest = serde_json::from_str(&json_content)?;
    Ok(manifest)
}
