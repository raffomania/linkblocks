use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImportFromOmnivore {
    pub api_token: String,
}
