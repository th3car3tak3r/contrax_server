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

// In src/server/core/manifest/manifest.rs

impl<'de> Deserialize<'de> for ContractDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 1. Unpack the outer step block (e.g. { "GET": { ... }, "->": { ... } } or just { "GET": { ... } })
        let mut raw_map = serde_json::Map::deserialize(deserializer)?;

        // 2. Safely extract the framework arrow transformation if it was placed at the sibling level
        let mut set_store_mappings = raw_map.remove("->").and_then(|v| match v {
            Value::Object(m) => Some(m),
            _ => None,
        });

        // 3. Find the real contract activation block (the key that isn't our internal tracking fields)
        let (id, inputs_val) = raw_map.into_iter().next().ok_or_else(|| {
            serde::de::Error::custom(
                "Pipeline step layout must declare a target contract invocation.",
            )
        })?;

        // 4. If the developer nested the "->" inside the contract block instead of next to it, extract it here
        let mut inner_obj = match inputs_val {
            Value::Object(obj) => obj,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Expected object configuration payload for contract key '{}'",
                    id
                )));
            }
        };

        if set_store_mappings.is_none() {
            if let Some(Value::Object(map)) = inner_obj.remove("->") {
                set_store_mappings = Some(map);
            }
        }

        // 5. Package the remaining items uniformly as standard input variables
        let inputs = inner_obj.into_iter().collect();

        Ok(ContractDefinition {
            id,
            inputs,
            set_store_mappings,
        })
    }
}

// In src/server/core/manifest/manifest.rs

#[derive(Deserialize, Debug, Clone)]
pub struct RouteDefinition {
    pub name: String,

    #[serde(default)]
    pub path: String,

    #[serde(default)]
    pub method: String,

    #[serde(alias = "pipeline")]
    pub contracts: Vec<ContractDefinition>,

    /// Matches either "output" or the minimalist rocket operator "=>"
    #[serde(default, alias = "=>")]
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
