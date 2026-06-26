use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DynamicContext {
    pub request: serde_json::Value,
    pub store: std::sync::Arc<tokio::sync::RwLock<serde_json::Map<String, serde_json::Value>>>,
    pub http_request: actix_web::HttpRequest,
}

impl DynamicContext {
    pub fn new(http_request: actix_web::HttpRequest, request_body: Value) -> Self {
        Self {
            request: request_body,
            store: Arc::new(RwLock::new(serde_json::Map::new())),
            http_request,
        }
    }
}
