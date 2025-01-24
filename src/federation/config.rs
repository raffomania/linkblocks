use activitypub_federation::config::FederationConfig;
use anyhow::{Context, Result};
use sqlx::PgPool;
use url::Url;

pub async fn new_config(
    db_pool: PgPool,
    base_url: Url,
) -> Result<FederationConfig<super::Context>> {
    let context = super::Context {
        db_pool,
        base_url: base_url.clone(),
    };
    FederationConfig::builder()
        .domain(
            base_url
                .domain()
                .context("Base URL must contain a domain name")?,
        )
        .app_data(context)
        .http_fetch_limit(1000)
        .debug(cfg!(debug_assertions))
        .build()
        .await
        .context("Failed to build activitypub config")
}
