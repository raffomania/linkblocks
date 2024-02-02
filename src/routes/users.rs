use askama_axum::IntoResponse;
use axum::{
    extract::Form,
    response::Redirect,
    response::Response,
    routing::{get, post},
    Router,
};
use garde::{Report, Validate};
use sqlx::{Pool, Postgres};
use tower_sessions::Session;

use crate::{
    app_error::AppResult,
    authentication::{self, AuthUser},
    db::Transaction,
    schemas::users::Credentials,
    views::users::LoginTemplate,
};

pub fn router() -> Router<Pool<Postgres>> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/logout", post(logout))
}

async fn post_login(
    Transaction(mut tx): Transaction,
    session: Session,
    Form(creds): Form<Credentials>,
) -> AppResult<Response> {
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

async fn get_login() -> AppResult<LoginTemplate> {
    Ok(LoginTemplate::default())
}

async fn logout(auth_user: AuthUser) -> AppResult<Redirect> {
    auth_user.logout().await?;
    Ok(Redirect::to("/login"))
}
