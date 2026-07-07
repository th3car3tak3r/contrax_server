// In src/server/contracts/web/fetch.rs

use crate::server::core::routing::Contract;
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Options configuration block per URL target used by the complex nested syntax
#[derive(Deserialize, Debug, Clone)]
pub struct FetchOptions {
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Value>,
}

/// Parameters for the flat/explicit configuration style
#[derive(Deserialize, Debug, Clone)]
pub struct ExplicitFetchParams {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Value>,
}

/// Container matching the: { "json": ... } wrapper structure.
/// Can be either a raw URL string shorthand or a full target configuration object.
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JsonPayload {
    /// Pattern A: "json": "https://jsonplaceholder.typicode.com/posts"
    Shorthand(String),
    /// Pattern B: "json": { "https://...": { "headers": {} } }
    Complex(HashMap<String, FetchOptions>),
}

/// Structural container matching the top-level format wrapper
#[derive(Deserialize, Debug, Clone)]
pub struct JsonFormatWrapper {
    pub json: JsonPayload,
}

/// The unified input enum that catches all three formatting variations seamlessly
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FetchInput {
    Nested(JsonFormatWrapper),
    Explicit(ExplicitFetchParams),
}

#[derive(Serialize, Debug, Clone)]
pub struct FetchOutput {
    pub status: u16,
    pub body: Value,
}

pub struct FetchContract {
    client: reqwest::Client,
    method: String,
}

impl FetchContract {
    pub fn new(method: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            method: method.to_uppercase(),
        }
    }
}

impl Contract for FetchContract {
    type Input = FetchInput;
    type Output = FetchOutput;

    fn validate(&self, input: &Self::Input) -> Result<(), String> {
        let url_str = match input {
            FetchInput::Nested(wrapper) => match &wrapper.json {
                JsonPayload::Shorthand(url) => url,
                JsonPayload::Complex(map) => map
                    .keys()
                    .next()
                    .ok_or("Fetch execution block missing nested target URL key.")?,
            },
            FetchInput::Explicit(params) => &params.url,
        };

        if reqwest::Url::parse(url_str).is_err() {
            return Err(format!("Invalid destination URL target: '{}'", url_str));
        }
        Ok(())
    }

    fn commit(&self, input: Self::Input) -> BoxFuture<'static, Result<Self::Output, String>> {
        let client = self.client.clone();
        let method_str = self.method.clone();

        async move {
            // Unify all structural input variants into flat primitives for execution
            let (url, headers, body) = match input {
                FetchInput::Nested(wrapper) => match wrapper.json {
                    JsonPayload::Shorthand(target_url) => (target_url, None, None),
                    JsonPayload::Complex(map) => {
                        let (target_url, options) = map
                            .into_iter()
                            .next()
                            .ok_or("Fetch execution block missing configuration target payload.")?;
                        (target_url, options.headers, options.body)
                    }
                },
                FetchInput::Explicit(params) => (params.url, params.headers, params.body),
            };

            // Build out the HTTP request worker uniformly
            let mut req_builder = match method_str.as_str() {
                "POST" => {
                    let payload = body.unwrap_or(Value::Null);
                    client.post(&url).json(&payload)
                }
                "PUT" => {
                    let payload = body.unwrap_or(Value::Null);
                    client.put(&url).json(&payload)
                }
                "DELETE" => client.delete(&url),
                _ => client.get(&url),
            };

            if let Some(headers_map) = headers {
                for (key, val) in headers_map {
                    req_builder = req_builder.header(key, val);
                }
            }

            let response = req_builder
                .send()
                .await
                .map_err(|e| format!("Network dispatch error: {}", e))?;

            let status = response.status().as_u16();
            let body: Value = response.json().await.unwrap_or(Value::Null);

            Ok(FetchOutput { status, body })
        }
        .boxed()
    }
}
