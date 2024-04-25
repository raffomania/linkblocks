use anyhow::Context;
use serde::{Deserialize, Serialize};

use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub nonce: Nonce,
    pub csrf_token: CsrfToken,
}

pub async fn client(
    base_url: String,
    oauth_google_client_id: String,
    oauth_google_client_secret: String,
) -> anyhow::Result<CoreClient> {
    let client_id = ClientId::new(oauth_google_client_id);
    let client_secret = ClientSecret::new(oauth_google_client_secret);
    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string())
        .context("failed to reach issuer")?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .context("failed to discover provider")?;
    // Set up the config for the Google OAuth2 process.
    let client =
        CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
            .set_redirect_uri(
                RedirectUrl::new(base_url + "/login_google_handler")
                    .context("Invalid redirect URL")?,
            );

    Ok(client)
}
