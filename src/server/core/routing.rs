pub mod build_contract_from_definition;
pub mod dynamic_context;
pub mod dynamic_pipeline;
pub mod dynamic_route;

use crate::server::core::routing::dynamic_context::DynamicContext;
use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;

pub trait DynamicContract: Send + Sync {
    fn execute(self: Arc<Self>, ctx: DynamicContext) -> BoxFuture<'static, Result<(), String>>;
}

pub trait InnerContract: Send + Sync {
    type Input: DeserializeOwned + Send + 'static;
    // 💡 FIXED: Stripped the Into<Value> constraint out
    type Output: Send + 'static;

    fn validate_pure(&self, input: &Self::Input) -> Result<(), String>;
    fn commit_pure(&self, input: Self::Input) -> BoxFuture<'static, Result<Self::Output, String>>;
}
