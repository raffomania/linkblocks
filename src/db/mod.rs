use anyhow::{Context, Result};
use sqlx::PgPool;

pub mod items;
pub mod links;
pub use links::LinkDestination;
pub use links::LinkDestinationWithChildren;
pub use links::LinkWithContent;
pub mod notes;
pub use notes::Note;
pub use notes::NoteWithLinks;
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
