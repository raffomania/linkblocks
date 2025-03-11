use anyhow::{anyhow, Context};
use openidconnect::url::Url;
use serde::{Deserialize, Serialize};

use openidconnect::core::{
    CoreClient, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType,
};
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    AccessTokenHash, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
};
use tower_sessions::Session;

use crate::cli::OidcArgs;
use crate::response_error::ResponseResult;

#[derive(Serialize, Deserialize)]
pub struct AuthenticatedOidcUserInfo {
    pub oidc_id: String,
    pub email: String,
}

impl AuthenticatedOidcUserInfo {
    const SESSION_KEY: &'static str = "oidc_user_info";

    pub async fn save_in_session(self, session: &Session) -> ResponseResult<()> {
        session
            .insert(Self::SESSION_KEY, self)
            .await
            .context("Failed to insert oidc data into session")?;

        Ok(())
    }

    pub async fn from_session(session: &Session) -> ResponseResult<Self> {
        Ok(session
            .get(Self::SESSION_KEY)
            .await
            .context("Failed to load oidc data from session")?
            .context("oidc data not found in session")?)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginAttempt {
    pub nonce: Nonce,
    pub csrf_token: CsrfToken,
    pub pkce_verifier: PkceCodeVerifier,
    pub authorize_url: Url,
}

impl LoginAttempt {
    const SESSION_KEY: &'static str = "oidc_login_attempt";

    pub fn new(client: &CoreClient) -> Self {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_token, nonce) = client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        LoginAttempt {
            nonce,
            csrf_token,
            pkce_verifier,
            authorize_url,
        }
    }

    pub async fn save_in_session(self, session: &Session) -> ResponseResult<()> {
        session
            .insert(Self::SESSION_KEY, self)
            .await
            .context("Failed to insert login attempt into session")?;

        Ok(())
    }

    pub async fn from_session(session: &Session) -> ResponseResult<Self> {
        Ok(session
            .get(Self::SESSION_KEY)
            .await
            .context("Failed to load oidc login attempt from session")?
            .context("oidc login attempt not found in session")?)
    }

    pub async fn login(
        self,
        oidc_client: &CoreClient,
        csrf_token: CsrfToken,
        code: AuthorizationCode,
    ) -> ResponseResult<AuthenticatedOidcUserInfo> {
        if csrf_token.secret() != self.csrf_token.secret() {
            return Err(anyhow!("CSRF token mismatch").into());
        }
        let token_response = oidc_client
            .clone()
            .exchange_code(code)
            .set_pkce_verifier(self.pkce_verifier)
            .request_async(async_http_client)
            .await
            .context("failed to get token response")?;
        let id_token_verifier: CoreIdTokenVerifier = oidc_client.id_token_verifier();
        let id_token = token_response
            .extra_fields()
            .id_token()
            .context("Server did not return an ID token")?;
        let id_token_claims = id_token
            .claims(&id_token_verifier, &self.nonce)
            .context("failed to get token claims")?;

        if let Some(expected_access_token_hash) = id_token_claims.access_token_hash() {
            let actual_access_token_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                &id_token
                    .signing_alg()
                    .context("failed to get signing algorithm")?,
            )
            .context("Failed to get access token hash from token response")?;
            if actual_access_token_hash != *expected_access_token_hash {
                return Err(anyhow!("Invalid access token").into());
            }
        }

        let email = id_token_claims
            .email()
            .context("failed to get email")?
            .to_string();

        let oidc_id = id_token_claims.subject().to_string();

        Ok(AuthenticatedOidcUserInfo { oidc_id, email })
    }
}

#[derive(Clone)]
pub struct Config {
    pub client: CoreClient,
    pub name: String,
}

// This enum is basically an Option with additional
// semantics. As such, it's expected that the
// `NotConfigured` variant will take up lots of space,
// Just like `None` would
#[expect(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum State {
    NotConfigured,
    Configured(Config),
}

impl State {
    #[must_use]
    pub fn get_client(self) -> Option<CoreClient> {
        match self {
            State::NotConfigured => None,
            State::Configured(Config { client, name: _ }) => Some(client),
        }
    }

    pub async fn initialize(base_url: String, args: Option<OidcArgs>) -> State {
        match Self::try_initialize_state(base_url, args).await {
            Ok(conf) => {
                tracing::info!("OIDC enabled.");
                State::Configured(conf)
            }
            Err(e) => {
                tracing::info!("OIDC disabled: {e:?}");
                State::NotConfigured
            }
        }
    }

    async fn try_initialize_state(
        base_url: String,
        args: Option<OidcArgs>,
    ) -> anyhow::Result<Config> {
        let args = args.context("OIDC configuration is absent or incomplete.")?;
        let client_id = ClientId::new(args.oidc_client_id);
        let client_secret = ClientSecret::new(args.oidc_client_secret);
        let issuer_url =
            IssuerUrl::new(args.oidc_issuer_url).context("failed to parse issuer URL")?;

        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .context("failed to discover provider")?;
        // Set up the config for the OIDC process.
        let client =
            CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
                .set_redirect_uri(
                    RedirectUrl::new(base_url + "/login_oidc_redirect")
                        .context("Invalid redirect URL")?,
                );

        Ok(Config {
            client,
            name: args.oidc_issuer_name,
        })
    }
}
