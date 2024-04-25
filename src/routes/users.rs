use anyhow::Context;
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
    forms::users::{Login, OidcLoginQuery},
    oidc,
    response_error::ResponseResult,
    server::AppState,
    views::{layout, login, users::ProfileTemplate},
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/login_oidc_redirect", get(get_login_oidc_redirect))
        .route("/login_oidc", get(get_login_oidc))
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
        return Ok(login::Template::new(errors, input, state.oidc_state).into_response());
    };

    let logged_in = authentication::login(&mut tx, session, &input.credentials).await;
    if logged_in.is_err() {
        let mut errors = Report::new();
        errors.append(
            garde::Path::new("root"),
            garde::Error::new("Username or password not correct"),
        );
        return Ok(login::Template::new(errors, input, state.oidc_state).into_response());
    }

    let redirect_to = input.previous_uri.unwrap_or("/".to_string());

    Ok(Redirect::to(&redirect_to).into_response())
}

async fn get_login_oidc(
    State(state): State<AppState>,
    session: Session,
) -> ResponseResult<Response> {
    // TODO: Store the CSRF and none states in a way that is more secure than this, although the current method is already quite secure.
    let client = state
        .oidc_state
        .get_client()
        .context("OIDC client not configured")?;
    let attempt = oidc::LoginAttempt::new(&client);
    let authorize_url = attempt.authorize_url.clone();
    attempt.save_in_session(&session).await?;

    Ok(Redirect::to(authorize_url.as_str()).into_response())
}

async fn get_login_oidc_redirect(
    session: Session,
    Query(query): Query<OidcLoginQuery>,
    state: State<AppState>,
    extract::Tx(mut tx): extract::Tx,
) -> ResponseResult<Response> {
    let oidc_client = state
        .oidc_state
        .clone()
        .get_client()
        .context("OIDC client not configured")?;

    authentication::login_oidc_user(&mut tx, &session, query, oidc_client).await?;

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
            state.oidc_state,
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
