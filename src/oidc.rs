use anyhow::Context;
use serde::{Deserialize, Serialize};

use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeVerifier, RedirectUrl,
};

use crate::server::{OauthConfig, OauthState};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub nonce: Nonce,
    pub csrf_token: CsrfToken,
    pub pkce_verifier: PkceCodeVerifier,
}

pub async fn init_oauth_state(
    base_url: String,
    oauth_client_id: Option<String>,
    oauth_client_secret: Option<String>,
    oauth_issuer_url: Option<String>,
    oauth_issuer_name: Option<String>,
) -> OauthState {
    let try_init_oauth_state = move || async {
        let (Some(id), Some(secret), Some(url), Some(name)) = (
            oauth_client_id,
            oauth_client_secret,
            oauth_issuer_url,
            oauth_issuer_name,
        ) else {
            anyhow::bail!("OIDC configuration is absent or incomplete.");
        };
        let client_id = ClientId::new(id);
        let client_secret = ClientSecret::new(secret);
        let issuer_url = IssuerUrl::new(url).context("failed to parse issuer URL")?;

        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .context("failed to discover provider")?;
        // Set up the config for the OAuth2 process.
        let client =
            CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
                .set_redirect_uri(
                    RedirectUrl::new(base_url + "/login_oauth_handler")
                        .context("Invalid redirect URL")?,
                );

        Ok(OauthConfig { client, name })
    };

    match try_init_oauth_state().await {
        Ok(conf) => {
            log::info!("OIDC enabled.");
            OauthState::Configured(conf)
        }
        Err(e) => {
            log::info!("OIDC disabled: {e:?}");
            OauthState::NotConfigured
        }
    }
}
