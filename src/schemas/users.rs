use serde::Deserialize;

pub struct CreateUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
