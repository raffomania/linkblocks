use crate::{
    app_error::{AppError, AppResult},
    db::{self, AppTx},
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

pub fn hash_password(password: String) -> AppResult<String> {
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    Ok(
        argon2::PasswordHasher::hash_password(&argon2, password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {e}"))?
            .to_string(),
    )
}

pub fn verify_password(user: &db::User, password: &str) -> AppResult<()> {
    let password_hash = &argon2::PasswordHash::new(&user.password_hash)
        .map_err(|e| anyhow!("Failed to create password hash: {e}"))?;

    argon2::Argon2::default()
        .verify_password(password.as_bytes(), password_hash)
        .map_err(|_e| AppError::NotAuthenticated)?;

    Ok(())
}

pub async fn login(tx: &mut AppTx, session: Session, creds: &Credentials) -> AppResult<()> {
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

    pub async fn save_in_session(session: &Session, id: &Uuid) -> AppResult<()> {
        session
            .insert(Self::SESSION_KEY, id)
            .await
            .context("Failed to insert id into session")?;

        Ok(())
    }

    pub async fn from_session(session: Session) -> AppResult<Self> {
        let user_id: Uuid = session
            .get("auth_user_id")
            .await
            .context("Failed to load authenticated user id")?
            .ok_or(AppError::NotAuthenticated)?;

        Ok(Self { user_id, session })
    }

    pub async fn logout(self) -> AppResult<()> {
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
            .map_err(|e| AppError::from(e).into_response())?;

        let auth_user = AuthUser::from_session(session).await;
        if let Err(AppError::NotAuthenticated) = auth_user {
            return Err(Redirect::to("/login").into_response());
        }

        Ok(auth_user.map_err(|e| e.into_response())?)
    }
}
