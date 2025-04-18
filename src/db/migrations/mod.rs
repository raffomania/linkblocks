//! Rust-based migrations.
//! These are interleaved with SQL-based migrations from the top-level
//! `migrations` folder.

use anyhow::Result;
use sqlx::PgTransaction;
use url::Url;

mod generate_missing_ap_users;

#[allow(clippy::inconsistent_digit_grouping)]
pub async fn run_before(
    previous_migration: &sqlx::migrate::Migration,
    tx: &mut PgTransaction<'_>,
    base_url: &Url,
) -> Result<()> {
    if previous_migration.version == 2025_10_14_160454 {
        generate_missing_ap_users::migrate(tx, base_url).await?;
    }
    Ok(())
}
