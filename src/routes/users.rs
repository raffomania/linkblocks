use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    response::{Redirect, Response},
    routing::{get, post},
    Router,
};
use garde::{Report, Validate};
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    authentication::{self, AuthUser},
    extract::{self, qs_form::QsForm},
    forms::users::Login,
    response_error::ResponseResult,
    server::AppState,
    views::{
        layout::LayoutTemplate,
        login::{DemoLoginTemplate, LoginTemplate},
        users::ProfileTemplate,
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
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
