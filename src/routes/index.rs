use crate::{app_error::Result, authentication::AuthUser};
use askama::Template;
use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get(index))
}

async fn index(_auth_user: AuthUser) -> Result<IndexTemplate> {
    Ok(IndexTemplate {})
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}
