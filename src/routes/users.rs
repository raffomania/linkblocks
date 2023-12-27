use askama::Template;
use axum::{debug_handler, response::Redirect, Form};
use tower_sessions::Session;

use crate::{app_error::Result, authentication, db::Transaction, schemas::users::Credentials};

#[debug_handler(state=sqlx::PgPool)]
pub async fn post_login(
    Transaction(mut tx): Transaction,
    session: Session,
    Form(creds): Form<Credentials>,
) -> Result<Redirect> {
    authentication::login(&mut tx, session, creds).await?;

    Ok(Redirect::to("/"))
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {}

#[debug_handler(state=sqlx::PgPool)]
pub async fn get_login() -> Result<LoginTemplate> {
    Ok(LoginTemplate {})
}
