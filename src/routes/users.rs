use anyhow::{anyhow, Context};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use garde::{Report, Validate};
use tower_sessions::Session;

use crate::{
    authentication::{self, AuthUser},
    db,
    extract::{self, qs_form::QsForm},
    forms::users::{CreateOidcUser, Login, OidcLoginQuery, OidcSelectUsername},
    oidc::{self},
    response_error::{ResponseError, ResponseResult},
    server::AppState,
    views::{self, layout, login, users::ProfileTemplate},
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/login_oidc_redirect", get(get_login_oidc_redirect))
        .route("/login_oidc_redirect", post(post_login_oidc_redirect))
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

    let oidc_session: oidc::LoginAttempt = oidc::LoginAttempt::from_session(&session).await?;
    let authed_oidc_info = oidc_session
        .login(&oidc_client, query.state, query.code)
        .await?;

    let existing_user = db::users::user_by_oidc_id(&mut tx, &authed_oidc_info.oidc_id).await;
    match existing_user {
        // Authenticate existing users in session
        Ok(existing_user) => {
            authentication::login_oidc_user(&session, &existing_user).await?;
            Ok(Redirect::to("/").into_response())
        }
        // Show new users a form to choose a username
        Err(ResponseError::NotFound) => {
            authed_oidc_info.save_in_session(&session).await?;
            Ok(views::oidc_select_username::Template::default().into_response())
        }
        Err(e) => Err(e),
    }
}

async fn post_login_oidc_redirect(
    session: Session,
    extract::Tx(mut tx): extract::Tx,
    QsForm(input): QsForm<OidcSelectUsername>,
) -> ResponseResult<Response> {
    if let Err(errors) = input.validate() {
        return Ok(views::oidc_select_username::Template {
            errors: errors.into(),
            input,
        }
        .into_response());
    };

    let authed_oidc_info = oidc::AuthenticatedOidcUserInfo::from_session(&session).await?;

    let create_oidc_user = CreateOidcUser {
        oidc_id: authed_oidc_info.oidc_id,
        email: authed_oidc_info.email,
        username: input.username,
    };

    if let Err(e) = create_oidc_user.validate() {
        return Err(anyhow!("Invalid OIDC user data received").context(e).into());
    }

    authentication::create_and_login_oidc_user(&mut tx, &session, create_oidc_user).await?;

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
    let layout = layout::Template::from_db(&mut tx, Some(&auth_user)).await?;

    Ok(ProfileTemplate {
        layout,
        base_url: state.base_url,
    })
}

async fn logout(auth_user: AuthUser) -> ResponseResult<Redirect> {
    auth_user.logout().await?;
    Ok(Redirect::to("/login"))
}
