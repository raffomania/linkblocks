use anyhow::{Context, Result};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::PgPool;

use crate::response_error::ResponseError;

pub mod links;
pub use links::LinkDestination;
pub use links::LinkWithContent;
pub mod lists;
pub use lists::List;
pub mod notes;
pub use notes::Note;
pub mod users;
pub use users::User;
pub mod bookmarks;
pub use bookmarks::Bookmark;

pub async fn migrate(pool: &PgPool) -> Result<()> {
    tracing::info!("Migrating the database...");
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("Database migrated.");

    Ok(())
}

pub async fn pool(url: &str) -> Result<sqlx::PgPool> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .context("Failed to create database connection pool")
}

pub type AppTx = sqlx::Transaction<'static, sqlx::Postgres>;

// TODO move into own file
pub struct ExtractTx(pub AppTx);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractTx
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ResponseError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.begin().await?;

        Ok(Self(conn))
    }
}
