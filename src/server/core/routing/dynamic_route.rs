use crate::server::core::manifest::manifest_resolver::resolve_manifest_value;
use crate::server::core::routing::dynamic_context::DynamicContext;
use crate::server::core::routing::dynamic_pipeline::DynamicPipeline;
use actix_web::{HttpResponse, Responder, web};
use serde_json::Value;
use std::collections::HashMap;

pub async fn dynamic_route(
    req: actix_web::HttpRequest,
    body: web::Bytes,
    pipeline: web::Data<DynamicPipeline>,
    output_mapping: web::Data<HashMap<String, Value>>,
) -> impl Responder {
    // 1. Start with our parsed JSON body payload (or an empty map)
    let mut request_map = match serde_json::from_slice::<Value>(&body) {
        Ok(Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };

    // 2. 💡 FIXED: Extract the named URL path parameters (e.g., from "/users/{value}")
    let mut path_map = serde_json::Map::new();
    for param in req.match_info().iter() {
        path_map.insert(param.0.to_string(), Value::String(param.1.to_string()));
    }
    // Nest them under "path" so your manifest can use "/path/value"
    request_map.insert("path".to_string(), Value::Object(path_map));

    // 3. Extract query string parameters (e.g., "?page_number=1") safely
    if let Ok(query_map) = web::Query::<HashMap<String, String>>::from_query(req.query_string()) {
        let mut query_json_map = serde_json::Map::new();
        for (key, val) in query_map.into_inner() {
            query_json_map.insert(key, Value::String(val));
        }
        request_map.insert("query".to_string(), Value::Object(query_json_map));
    }

    // Wrap our unified payload into a generic JSON Value container
    let request_json = Value::Object(request_map);

    // 4. Initialize our dynamic execution context carrying the populated data structure
    let ctx = DynamicContext {
        request: request_json,
        store: std::sync::Arc::new(tokio::sync::RwLock::new(serde_json::Map::new())),
        http_request: req,
    };

    // 5. Run the dynamic sequence steps safely
    if let Err(pipeline_err) = pipeline.execute(ctx.clone()).await {
        return HttpResponse::InternalServerError()
            .body(format!("Pipeline Fault: {}", pipeline_err));
    }

    // 6. Extract safe, isolated thread boundaries before evaluation loop
    let request_data = &ctx.request;
    let pipeline_store = &ctx.store;

    // 7. Transform the context store into our custom declarative shape
    let mut response_payload = serde_json::Map::new();

    for (target_field_name, rule_value) in output_mapping.get_ref() {
        let final_val = resolve_manifest_value(rule_value, request_data, pipeline_store).await;

        response_payload.insert(target_field_name.clone(), final_val);
    }

    // 8. Return the mapped payload back to the client
    HttpResponse::Ok().json(Value::Object(response_payload))
}
