pub mod build_contract_from_definition;
pub mod dynamic_context;
pub mod dynamic_pipeline;
pub mod dynamic_route;

use crate::server::core::routing::dynamic_context::DynamicContext;
use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub trait DynamicContract: Send + Sync {
    fn execute(self: Arc<Self>, ctx: DynamicContext) -> BoxFuture<'static, Result<(), String>>;
}

pub trait Contract: Send + Sync {
    type Input: DeserializeOwned + Send + 'static;
    type Output: Send + 'static;

    fn validate(&self, input: &Self::Input) -> Result<(), String>;
    fn commit(&self, input: Self::Input) -> BoxFuture<'static, Result<Self::Output, String>>;
}
