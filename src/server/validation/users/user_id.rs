use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UserId {
    #[validate(length(equal = 12, message = "Invalid user id length"))]
    pub value: String,
}
