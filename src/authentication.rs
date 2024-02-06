use crate::{
    db::{self, AppTx},
    response_error::{ResponseError, ResponseResult},
    schemas::users::Credentials,
};
use anyhow::{anyhow, Context};
use argon2::PasswordVerifier;
use askama_axum::IntoResponse;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    response::{Redirect, Response},
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

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
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

    pub async fn from_session(session: Session) -> ResponseResult<Self> {
        let user_id: Uuid = session
            .get("auth_user_id")
            .await
            .context("Failed to load authenticated user id")?
            .ok_or(ResponseError::NotAuthenticated)?;

        Ok(Self { user_id, session })
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
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        req: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state)
            .await
            .map_err(|_| anyhow!("Failed to extract session"))
            .map_err(|e| ResponseError::from(e).into_response())?;

        let auth_user = AuthUser::from_session(session).await;
        if let Err(ResponseError::NotAuthenticated) = auth_user {
            return Err(Redirect::to("/login").into_response());
        }

        Ok(auth_user.map_err(|e| e.into_response())?)
    }
}
