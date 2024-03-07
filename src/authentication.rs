use crate::{
    db::{self, AppTx},
    extract,
    forms::users::{CreateUser, Credentials},
    response_error::{ResponseError, ResponseResult},
    server::AppState,
};
use anyhow::{anyhow, Context};
use argon2::PasswordVerifier;
use askama::filters::urlencode;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, OriginalUri},
    http::request::Parts,
    response::Redirect,
};
use tower_sessions::Session;
use uuid::Uuid;

pub fn hash_password(password: String) -> ResponseResult<String> {
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    Ok(
        argon2::PasswordHasher::hash_password(&argon2, password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {e}"))?
            .to_string(),
    )
}

pub fn verify_password(user: &db::User, password: &str) -> ResponseResult<()> {
    let password_hash = &argon2::PasswordHash::new(&user.password_hash)
        .map_err(|e| anyhow!("Failed to create password hash: {e}"))?;

    argon2::Argon2::default()
        .verify_password(password.as_bytes(), password_hash)
        .map_err(|_e| ResponseError::NotAuthenticated)?;

    Ok(())
}

pub async fn login(tx: &mut AppTx, session: Session, creds: &Credentials) -> ResponseResult<()> {
    let user = db::users::by_username(tx, &creds.username).await?;

    verify_password(&user, &creds.password)?;

    AuthUser::save_in_session(&session, &user.id).await?;

    Ok(())
}

pub async fn create_and_login_temp_user(tx: &mut AppTx, session: Session) -> ResponseResult<()> {
    let username =
        friendly_zoo::Zoo::new(friendly_zoo::Species::CustomDelimiter(' '), 1).generate();
    let password = Uuid::new_v4().to_string();
    let user = db::users::insert(tx, CreateUser { password, username }).await?;

    AuthUser::save_in_session(&session, &user.id).await?;

    Ok(())
}

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub user: db::User,
    session: Session,
}

impl AuthUser {
    const SESSION_KEY: &'static str = "auth_user_id";

    pub async fn save_in_session(session: &Session, id: &Uuid) -> ResponseResult<()> {
        session
            .insert(Self::SESSION_KEY, id)
            .await
            .context("Failed to insert id into session")?;

        Ok(())
    }

    pub async fn from_session(session: Session, tx: &mut AppTx) -> ResponseResult<Self> {
        let user_id: Uuid = session
            .get("auth_user_id")
            .await
            .context("Failed to load authenticated user id")?
            .ok_or(ResponseError::NotAuthenticated)?;

        let user = db::users::by_id(tx, user_id).await?;

        Ok(Self {
            user_id,
            user,
            session,
        })
    }

    pub async fn logout(self) -> ResponseResult<()> {
        self.session
            .remove::<Uuid>(Self::SESSION_KEY)
            .await
            .context("Failed to remove user id from session")?;
        Ok(())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Redirect;

    async fn from_request_parts(
        req: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let uri = OriginalUri::from_request_parts(req, state).await.unwrap();

        let redirect_after_login = uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or_default();
        let redirect_after_login = urlencode(redirect_after_login).unwrap_or_default();

        let redirect_to = format!("/login?previous_uri={redirect_after_login}",);
        let error_redirect = Redirect::to(&redirect_to);

        let session = Session::from_request_parts(req, state).await.map_err(|e| {
            tracing::error!("{e:?}");
            error_redirect.clone()
        })?;

        let extract::Tx(mut tx) =
            extract::Tx::from_request_parts(req, state)
                .await
                .map_err(|e| {
                    tracing::error!("{e:?}");
                    error_redirect.clone()
                })?;

        let auth_user = AuthUser::from_session(session, &mut tx).await;
        if let Err(ResponseError::NotAuthenticated) = auth_user {
            return Err(error_redirect);
        }

        Ok(auth_user.map_err(|e| {
            tracing::error!("{e:?}");
            error_redirect
        })?)
    }
}
