use crate::server::core::routing::InnerContract;
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

/// The explicit data fields required by this contract to run safely
#[derive(Deserialize, Serialize, Debug, Clone, Validate)]
pub struct Input {
    #[validate(length(
        min = 1,
        max = 6,
        message = "UserId must be between 1 and 36 characters"
    ))]
    pub user_id: String,
}

pub struct Create;

impl InnerContract for Create {
    type Input = Input;
    type Output = Value;

    fn validate_pure(&self, input: &Self::Input) -> Result<(), String> {
        input
            .validate()
            .map_err(|errors| format!("Validation failed: {}", errors))
    }

    fn commit_pure(&self, input: Self::Input) -> BoxFuture<'static, Result<Self::Output, String>> {
        async move {
            println!("create user fired");
            Ok(Value::Null)
        }
        .boxed()
    }
}
