use crate::{app_error::Result, authentication::AuthUser};
use askama::Template;
use axum::debug_handler;

#[debug_handler(state=sqlx::PgPool)]
pub async fn index(_auth_user: AuthUser) -> Result<IndexTemplate> {
    Ok(IndexTemplate {})
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}
