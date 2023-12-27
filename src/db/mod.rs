use anyhow::{Context, Result};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::PgPool;

use crate::app_error::AppError;

pub mod users;
pub use users::User;

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

pub struct Transaction(pub sqlx::Transaction<'static, sqlx::Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for Transaction
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.begin().await?;

        Ok(Self(conn))
    }
}
