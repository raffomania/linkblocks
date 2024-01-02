use askama::Template;
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
    app_error::Result,
    authentication::{self, AuthUser},
    db::Transaction,
    form_errors::FormErrors,
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
) -> Result<Response> {
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

#[derive(Template, Default)]
#[template(path = "login.html")]
struct LoginTemplate {
    errors: FormErrors,
    input: Credentials,
}

impl LoginTemplate {
    fn new(errors: Report, input: Credentials) -> Self {
        Self {
            errors: errors.into(),
            input: Credentials {
                username: input.username,
                // Never render the password we got from the user
                password: String::new(),
            },
        }
    }
}

async fn get_login() -> Result<LoginTemplate> {
    Ok(LoginTemplate::default())
}

async fn logout(auth_user: AuthUser) -> Result<Redirect> {
    auth_user.logout().await?;
    Ok(Redirect::to("/login"))
}
