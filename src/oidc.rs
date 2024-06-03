use anyhow::Context;
use tokio::task;

use crate::db::oidc::GoogleProviderMetadata;

use openidconnect::core::CoreClient;
use openidconnect::reqwest::http_client;
use openidconnect::{ClientId, ClientSecret, IssuerUrl, RedirectUrl, RevocationUrl};

pub async fn oidc_client(
    base_url: String,
    oauth_google_client_id: String,
    oauth_google_client_secret: String,
) -> CoreClient {
    let google_client_id = ClientId::new(oauth_google_client_id);
    let google_client_secret = ClientSecret::new(oauth_google_client_secret);
    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string())
        .context("failed to reach issuer")
        .unwrap();

    let provider_metadata = task::spawn_blocking(move || {
        GoogleProviderMetadata::discover(&issuer_url, http_client)
            .context("failed to discover provider")
            .unwrap()
    })
    .await
    .context("failed to discover provider")
    .unwrap();

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();
    // Set up the config for the Google OAuth2 process.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    .set_redirect_uri(
        RedirectUrl::new(base_url + "/login_google_handler").expect("Invalid redirect URL"),
    )
    .set_revocation_uri(
        RevocationUrl::new(revocation_endpoint).expect("Invalid revocation endpoint URL"),
    );
    return client;
}
