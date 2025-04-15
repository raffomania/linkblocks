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
    let domain = base_url
        .domain()
        .context("Base URL must contain a domain name")?;
    let port = base_url.port().map_or(String::new(), |p| format!(":{p}"));
    FederationConfig::builder()
        .domain(format!("{domain}{port}"))
        .app_data(context)
        .http_fetch_limit(1000)
        .debug(cfg!(debug_assertions))
        .build()
        .await
        .context("Failed to build activitypub config")
}
