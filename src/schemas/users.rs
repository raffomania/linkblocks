use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Validate)]
pub struct CreateUser {
    #[garde(alphanumeric, ascii, length(min = 3, max = 50))]
    pub username: String,
    #[garde(length(min = 10, max = 100))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate, Debug, Default)]
pub struct Credentials {
    #[garde(alphanumeric, ascii, length(min = 3, max = 50))]
    pub username: String,
    #[garde(length(min = 10, max = 100))]
    pub password: String,
}
