use serde::{Deserialize, Serialize};

pub struct CreateUser {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
