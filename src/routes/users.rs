use askama::Template;
use axum::{
    response::Redirect,
    routing::{get, post},
    Form, Router,
};
use sqlx::{Pool, Postgres};
use tower_sessions::Session;

use crate::{
    app_error::Result,
    authentication::{self, AuthUser},
    db::Transaction,
    schemas::users::Credentials,
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
) -> Result<Redirect> {
    authentication::login(&mut tx, session, creds).await?;

    Ok(Redirect::to("/"))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

async fn get_login() -> Result<LoginTemplate> {
    Ok(LoginTemplate {})
}

async fn logout(auth_user: AuthUser) -> Result<Redirect> {
    auth_user.logout().await?;
    Ok(Redirect::to("/login"))
}
