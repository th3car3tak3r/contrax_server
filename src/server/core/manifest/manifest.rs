// In src/server/core/manifest/manifest.rs

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ContractDefinition {
    pub id: String,
    pub inputs: HashMap<String, Value>,
    pub set_store_mappings: Option<serde_json::Map<String, Value>>,
}

impl<'de> Deserialize<'de> for ContractDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut raw_map = HashMap::<String, Value>::deserialize(deserializer)?;

        let set_store_mappings = if let Some(Value::Object(map)) = raw_map.remove("set_store") {
            Some(map)
        } else {
            None
        };

        let (id, inputs_val) = raw_map.into_iter().next().ok_or_else(|| {
            serde::de::Error::custom(
                "Pipeline step layout must declare a target contract invocation.",
            )
        })?;

        let inputs = match inputs_val {
            Value::Object(obj) => obj.into_iter().collect(),
            _ => HashMap::new(),
        };

        Ok(ContractDefinition {
            id,
            inputs,
            set_store_mappings,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RouteDefinition {
    pub name: String,

    #[serde(default)]
    pub path: String,

    #[serde(default)]
    pub method: String,

    #[serde(alias = "pipeline")]
    pub contracts: Vec<ContractDefinition>,

    /// Fallback gracefully if a route doesn't explicitly declare a structural output layout block
    #[serde(default)]
    pub output: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerManifest {
    pub server_name: String,
    pub version: String,
    pub routes: Vec<RouteDefinition>,
}

pub fn load_manifest_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<ServerManifest, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut json_content = String::new();
    file.read_to_string(&mut json_content)?;

    let manifest: ServerManifest = serde_json::from_str(&json_content)?;
    Ok(manifest)
}
