use askama_axum::IntoResponse;
use axum::{
    extract::{Form, State},
    response::{Redirect, Response},
    routing::{get, post},
    Router,
};
use garde::{Report, Validate};
use tower_sessions::Session;

use crate::{
    authentication::{self, AuthUser},
    extract,
    forms::users::Credentials,
    response_error::ResponseResult,
    server::AppState,
    views::{layout::LayoutTemplate, login::LoginTemplate, users::ProfileTemplate},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/logout", post(logout))
        .route("/profile", get(get_profile))
}

async fn post_login(
    extract::Tx(mut tx): extract::Tx,
    session: Session,
    Form(creds): Form<Credentials>,
) -> ResponseResult<Response> {
    if let Err(errors) = creds.validate(&()) {
        return Ok(LoginTemplate::new(errors, creds).into_response());
    };

    let logged_in = authentication::login(&mut tx, session, &creds).await;
    if logged_in.is_err() {
        let mut errors = Report::new();
        errors.append(
            garde::Path::new("root"),
            garde::Error::new("Username or password not correct"),
        );
        return Ok(LoginTemplate::new(errors, creds).into_response());
    }

    Ok(Redirect::to("/").into_response())
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

async fn get_login() -> ResponseResult<LoginTemplate> {
    Ok(LoginTemplate::default())
}

async fn logout(auth_user: AuthUser) -> ResponseResult<Redirect> {
    auth_user.logout().await?;
    Ok(Redirect::to("/login"))
}
