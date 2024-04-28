use anyhow::Context;
use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State}, response::{Redirect, Response}, routing::{get, post}, Router
};
use tokio::task;
use garde::{Report, Validate};
use tower_sessions::Session;

use crate::{
    authentication::{self, AuthUser}, extract::{self, qs_form::QsForm}, forms::users::{Login, CreateOAuthUser}, response_error::ResponseResult, server::AppState, views::{
        layout::LayoutTemplate,
        login::{DemoLoginTemplate, LoginTemplate},
        users::ProfileTemplate,
    }
};
use serde::{Deserialize, Serialize};

use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreGrantType,
    CoreIdTokenVerifier, CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse,
    CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm,
    CoreResponseMode, CoreResponseType, CoreRevocableToken, CoreSubjectIdentifierType,
};
use openidconnect::reqwest::http_client;
use openidconnect::{
    AdditionalProviderMetadata, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, ProviderMetadata, RedirectUrl, RevocationUrl,
    Scope,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}
impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}
type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;
#[derive(Serialize, Deserialize)]
struct OidcSession {
    nonce: Nonce,
    csrf_token: CsrfToken,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/login_google_handler", get(get_login_google_handler ))
        .route("/login_google", get(get_login_google ))
        .route("/login_demo", post(post_login_demo))
        .route("/logout", post(logout))
        .route("/profile", get(get_profile))
}


async fn post_login(
    extract::Tx(mut tx): extract::Tx,
    session: Session,
    QsForm(input): QsForm<Login>,
) -> ResponseResult<Response> {
    if let Err(errors) = input.validate() {
        return Ok(LoginTemplate::new(errors, input).into_response());
    };

    let logged_in = authentication::login(&mut tx, session, &input.credentials).await;
    if logged_in.is_err() {
        let mut errors = Report::new();
        errors.append(
            garde::Path::new("root"),
            garde::Error::new("Username or password not correct"),
        );
        return Ok(LoginTemplate::new(errors, input).into_response());
    }

    let redirect_to = input.previous_uri.unwrap_or("/".to_string());

    Ok(Redirect::to(&redirect_to).into_response())
}

async fn get_login_google(
    State(state): State<AppState>,
    session: Session,
) -> ResponseResult<Response> {

    let google_client_id = ClientId::new(
        state.oauth_google_client_id
    );
    let google_client_secret = ClientSecret::new(
        state.oauth_google_client_secret
    );
    let issuer_url = 
        IssuerUrl::new("https://accounts.google.com".to_string()).context("failed to reach issuer")?;

    let provider_metadata = task::spawn_blocking(move || {
        GoogleProviderMetadata::discover(&issuer_url, http_client)
        .context("failed to discover provider").unwrap()
    }).await.context("failed to discover provider")?;
    
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
        RedirectUrl::new(state.base_url.to_string() + "/login_google_handler").expect("Invalid redirect URL"),
    )
    .set_revocation_uri(
        RevocationUrl::new(revocation_endpoint).expect("Invalid revocation endpoint URL"),
    );

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string())).url();

    // TODO: Store the CSRF and none states in a way that is more secure than this, although the current method is already quire secure.

    session.insert("google_oidc_session", OidcSession { nonce, csrf_token:csrf_state }).await.context("failed to insert session")?;


    Ok(Redirect::to(authorize_url.as_str()).into_response())
}

#[derive(Deserialize)]
struct OAuthLoginQuery{
    code: String,
    state: String,
}

async fn get_login_google_handler(
    session: Session,
    Query(query): Query<OAuthLoginQuery>,
    state: State<AppState>,
    extract::Tx(mut tx): extract::Tx,
) -> ResponseResult<Response> {
    // get the nonce and csrf token from session
    let oidc_session: OidcSession = session.get("google_oidc_session").await.context("failed to get session")?.context("session not found")?;

    let code = AuthorizationCode::new(query.code.clone());
    let csrf_state = CsrfToken::new(query.state.clone());

    if csrf_state.secret() != oidc_session.csrf_token.secret() {
        return Ok("Invalid CSRF token".into_response());
    }
    let google_client_id = ClientId::new(
        state.oauth_google_client_id.clone()
    );
    let google_client_secret = ClientSecret::new(
        state.oauth_google_client_secret.clone()
    );
    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).context("failed to reach issuer")?;

    let provider_metadata = task::spawn_blocking(move || {
        GoogleProviderMetadata::discover(&issuer_url, http_client)
        .context("failed to discover provider").unwrap()
    }).await.context("failed to discover provider")?;
    
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
        RedirectUrl::new(state.base_url.to_string() + "/login_google_handler").expect("Invalid redirect URL"),
    )
    .set_revocation_uri(
        RevocationUrl::new(revocation_endpoint).expect("Invalid revocation endpoint URL"),
    );

    let id_token_claims = task::spawn_blocking(move || {
        let token_response = client.clone()
        .exchange_code(code)
        .request(http_client)
        .context("failed to get token response").unwrap();
        let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
        let token_claims = token_response
            .extra_fields()
            .id_token()
            .expect("Server did not return an ID token")
            .claims(&id_token_verifier, &oidc_session.nonce)
            .context("failed to get token claims").unwrap().clone();
        let token_to_revoke: CoreRevocableToken = match token_response.refresh_token() {
            Some(token) => token.into(),
            None => token_response.access_token().into(),
        };
    
        client
            .revoke_token(token_to_revoke)
            .expect("no revocation_uri configured")
            .request(http_client)
            .context("failed to revoke token").unwrap();
        token_claims
    }).await.context("failed to get token claims")?;

    let email = id_token_claims.email().context("failed to get email")?.to_string();
    // take username from email
    let username = email.split("@").next().context("failed to parse username")?.to_string();

    let oauth_id = id_token_claims.subject().to_string();

    let oauth_credentials = CreateOAuthUser {
        username,
        oauth_id,
        email,
        oauth_provider: "Google".to_string(),
    };
    let logged_in = authentication::login_oauth_user(&mut tx, session.clone(), &oauth_credentials.oauth_id).await;
    if logged_in.is_err() {
        let signed_up = authentication::create_and_login_oauth_user(&mut tx, session, oauth_credentials).await;
        if signed_up.is_err() {
            return Ok("Failed to login".into_response());
        }
    }
    tx.commit().await?;
    Ok(Redirect::to("/").into_response())
}

async fn post_login_demo(
    extract::Tx(mut tx): extract::Tx,
    session: Session,
) -> ResponseResult<Response> {
    authentication::create_and_login_temp_user(&mut tx, session).await?;
    tx.commit().await?;

    Ok(Redirect::to("/").into_response())
}

#[derive(Deserialize)]
struct LoginQuery {
    previous_uri: Option<String>,
}

async fn get_login(
    Query(query): Query<LoginQuery>,
    State(state): State<AppState>,
) -> ResponseResult<Response> {
    match state.demo_mode {
        true => Ok(DemoLoginTemplate {}.into_response()),
        false => Ok(LoginTemplate::new(
            Report::new(),
            Login {
                previous_uri: query.previous_uri,
                ..Default::default()
            },
        )
        .into_response()),
    }
}

async fn get_profile(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> ResponseResult<ProfileTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    Ok(ProfileTemplate {
        layout,
        base_url: state.base_url,
    })
}

async fn logout(auth_user: AuthUser) -> ResponseResult<Redirect> {
    auth_user.logout().await?;
    Ok(Redirect::to("/login"))
}
