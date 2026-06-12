use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone, Copy)]
pub struct PageNumber {
    #[serde(
        alias = "page_number",
        deserialize_with = "deserialize_number_from_string"
    )]
    #[validate(range(min = 1, message = "Page number must be 1 or greater"))]
    pub value: u32,
}
