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
pub struct Login {
    #[garde(skip)]
    pub previous_uri: Option<String>,
    #[garde(dive)]
    pub credentials: Credentials,
}

#[derive(Serialize, Deserialize, Validate, Debug, Default)]
pub struct Credentials {
    #[garde(alphanumeric, ascii, length(min = 3, max = 50))]
    pub username: String,
    #[garde(length(min = 10, max = 100))]
    pub password: String,
}
