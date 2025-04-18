//! Rust-based migrations.
//! These are interleaved with SQL-based migrations from the top-level
//! `migrations` folder.

use anyhow::Result;
use sqlx::PgTransaction;
use url::Url;

#[allow(clippy::inconsistent_digit_grouping)]
pub async fn run_before(
    previous_migration: &sqlx::migrate::Migration,
    tx: &mut PgTransaction<'_>,
    base_url: &Url,
) -> Result<()> {
    Ok(())
}
