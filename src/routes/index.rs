use crate::{app_error::Result, db::Transaction};
use askama::Template;
use axum::debug_handler;

#[debug_handler(state=sqlx::PgPool)]
pub async fn index(Transaction(mut tx): Transaction) -> Result<IndexTemplate> {
    let users = sqlx::query!("select count(*) from users;")
        .fetch_one(&mut *tx)
        .await?;
    dbg!(users);
    Ok(IndexTemplate {})
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}
