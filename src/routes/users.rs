use askama::Template;
use axum::{response::Redirect, routing::get, Form, Router};
use sqlx::{Pool, Postgres};
use tower_sessions::Session;

use crate::{app_error::Result, authentication, db::Transaction, schemas::users::Credentials};

pub fn router() -> Router<Pool<Postgres>> {
    Router::new().route("/login", get(get_login).post(post_login))
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
