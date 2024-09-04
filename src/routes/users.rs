use anyhow::{anyhow, Context};
use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    response::{Redirect, Response},
    routing::{get, post},
    Router,
};
use garde::{Report, Validate};
use tower_sessions::Session;

use crate::{
    authentication::{self, AuthUser},
    extract::{self, qs_form::QsForm},
    forms::users::{CreateOAuthUser, Login},
    oidc,
    response_error::ResponseResult,
    server::AppState,
    views::{layout, login, users::ProfileTemplate},
};
use serde::Deserialize;

use openidconnect::reqwest::async_http_client;
use openidconnect::{
    core::{CoreIdTokenVerifier, CoreResponseType},
    PkceCodeChallenge,
};
use openidconnect::{AuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, Scope};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/login_oauth_handler", get(get_login_oauth_handler))
        .route("/login_oauth", get(get_login_oauth))
        .route("/login_demo", post(post_login_demo))
        .route("/logout", post(logout))
        .route("/profile", get(get_profile))
}

async fn post_login(
    extract::Tx(mut tx): extract::Tx,
    session: Session,
    State(state): State<AppState>,
    QsForm(input): QsForm<Login>,
) -> ResponseResult<Response> {
    if let Err(errors) = input.validate() {
        return Ok(login::Template::new(errors, input, state.oauth_state).into_response());
    };

    let logged_in = authentication::login(&mut tx, session, &input.credentials).await;
    if logged_in.is_err() {
        let mut errors = Report::new();
        errors.append(
            garde::Path::new("root"),
            garde::Error::new("Username or password not correct"),
        );
        return Ok(login::Template::new(errors, input, state.oauth_state).into_response());
    }

    let redirect_to = input.previous_uri.unwrap_or("/".to_string());

    Ok(Redirect::to(&redirect_to).into_response())
}

async fn get_login_oauth(
    State(state): State<AppState>,
    session: Session,
) -> ResponseResult<Response> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state, nonce) = state
        .oauth_state
        .get_client()
        .context("Google OAuth client not configured")?
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // TODO: Store the CSRF and none states in a way that is more secure than this, although the current method is already quire secure.

    session
        .insert(
            "google_oidc_session",
            oidc::Session {
                nonce,
                csrf_token: csrf_state,
                pkce_verifier,
            },
        )
        .await
        .context("failed to insert session")?;

    Ok(Redirect::to(authorize_url.as_str()).into_response())
}

#[derive(Deserialize)]
struct OAuthLoginQuery {
    code: String,
    state: String,
}

async fn get_login_oauth_handler(
    session: Session,
    Query(query): Query<OAuthLoginQuery>,
    state: State<AppState>,
    extract::Tx(mut tx): extract::Tx,
) -> ResponseResult<Response> {
    // get the nonce and csrf token from session
    let oidc_session: oidc::Session = session
        .get("google_oidc_session")
        .await
        .context("failed to get session")?
        .context("session not found")?;

    let code = AuthorizationCode::new(query.code.clone());
    let csrf_state = CsrfToken::new(query.state.clone());

    if csrf_state.secret() != oidc_session.csrf_token.secret() {
        return Err(anyhow!("CSRF token mismatch").into());
    }

    let id_token_claims = {
        let oauth_google_client = state
            .oauth_state
            .clone()
            .get_client()
            .context("Google OAuth client not configured")?;
        let token_response = oauth_google_client
            .clone()
            .exchange_code(code)
            .set_pkce_verifier(oidc_session.pkce_verifier)
            .request_async(async_http_client)
            .await
            .context("failed to get token response")?;
        let id_token_verifier: CoreIdTokenVerifier = oauth_google_client.id_token_verifier();
        token_response
            .extra_fields()
            .id_token()
            .context("Server did not return an ID token")?
            .claims(&id_token_verifier, &oidc_session.nonce)
            .context("failed to get token claims")?
            .clone()
    };

    let email = id_token_claims
        .email()
        .context("failed to get email")?
        .to_string();

    let oauth_id = id_token_claims.subject().to_string();

    let oauth_credentials = CreateOAuthUser {
        oauth_id,
        email,
        oauth_provider: "Google".to_string(),
    };
    let logged_in =
        authentication::login_oauth_user(&mut tx, session.clone(), &oauth_credentials.oauth_id)
            .await;
    if logged_in.is_err() {
        let signed_up =
            authentication::create_and_login_oauth_user(&mut tx, session, oauth_credentials).await;
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
    if state.demo_mode {
        Ok(login::DemoTemplate {}.into_response())
    } else {
        Ok(login::Template::new(
            Report::new(),
            Login {
                previous_uri: query.previous_uri,
                ..Default::default()
            },
            state.oauth_state,
        )
        .into_response())
    }
}

async fn get_profile(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> ResponseResult<ProfileTemplate> {
    let layout = layout::Template::from_db(&mut tx, &auth_user).await?;

    Ok(ProfileTemplate {
        layout,
        base_url: state.base_url,
    })
}

async fn logout(auth_user: AuthUser) -> ResponseResult<Redirect> {
    auth_user.logout().await?;
    Ok(Redirect::to("/login"))
}
