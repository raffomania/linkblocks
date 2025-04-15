use anyhow::{Context, anyhow};
use argon2::PasswordVerifier;
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, OriginalUri},
    http::request::Parts,
    response::Redirect,
};
use percent_encoding::utf8_percent_encode;
use tower_sessions::Session;
use url::Url;
use uuid::Uuid;

use crate::{
    db::{self, AppTx, User},
    forms::users::{CreateOidcUser, CreateUser, Credentials},
    response_error::{ResponseError, ResponseResult},
    server::AppState,
};

pub fn hash_password(password: &String) -> ResponseResult<String> {
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
    let existing_hash = user
        .password_hash
        .as_ref()
        .context("User has no password set")?;
    let password_hash = &argon2::PasswordHash::new(existing_hash)
        .map_err(|e| anyhow!("Failed to create password hash: {e}"))?;

    argon2::Argon2::default()
        .verify_password(password.as_bytes(), password_hash)
        .map_err(|_e| ResponseError::NotAuthenticated)?;

    Ok(())
}

pub async fn login(tx: &mut AppTx, session: Session, creds: &Credentials) -> ResponseResult<()> {
    let user = db::users::by_username(tx, &creds.username).await?;

    verify_password(&user, &creds.password)?;

    AuthUser::save_in_session(&session, user.id).await?;

    Ok(())
}

pub async fn create_and_login_temp_user(
    tx: &mut AppTx,
    session: Session,
    base_url: &Url,
) -> ResponseResult<()> {
    let username =
        friendly_zoo::Zoo::new(friendly_zoo::Species::CustomDelimiter(' '), 1).generate();
    let password = Uuid::new_v4().to_string();
    let user = db::users::insert(tx, CreateUser { username, password }, base_url).await?;

    AuthUser::save_in_session(&session, user.id).await?;

    Ok(())
}

pub async fn create_and_login_oidc_user(
    tx: &mut AppTx,
    session: &Session,
    create_oidc_user: CreateOidcUser,
) -> ResponseResult<()> {
    let user = db::users::user_by_oidc_id(tx, &create_oidc_user.oidc_id).await;

    let user = match user {
        Ok(user) => user,
        Err(ResponseError::NotFound) => db::users::insert_oidc(tx, create_oidc_user).await?,
        Err(_) => return Err(anyhow!("Failed to look up user by OIDC id").into()),
    };

    AuthUser::save_in_session(session, user.id).await?;

    Ok(())
}

pub async fn login_oidc_user(session: &Session, user: &User) -> ResponseResult<()> {
    AuthUser::save_in_session(session, user.id).await
}

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    session: Session,
}

impl AuthUser {
    const SESSION_KEY: &'static str = "auth_user_id";

    pub async fn save_in_session(session: &Session, id: Uuid) -> ResponseResult<()> {
        session
            .insert(Self::SESSION_KEY, id)
            .await
            .context("Failed to insert id into session")?;

        Ok(())
    }

    pub async fn from_session(session: Session) -> ResponseResult<Self> {
        let user_id: Uuid = session
            .get(Self::SESSION_KEY)
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

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Redirect;

    async fn from_request_parts(
        req: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let uri = OriginalUri::from_request_parts(req, state).await.unwrap();

        let redirect_after_login = uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or_default();
        let redirect_after_login =
            utf8_percent_encode(redirect_after_login, percent_encoding::NON_ALPHANUMERIC)
                .to_string();

        let redirect_to = format!("/login?previous_uri={redirect_after_login}",);
        let error_redirect = Redirect::to(&redirect_to);

        let session = Session::from_request_parts(req, state).await.map_err(|e| {
            tracing::error!("Failed to initialize session: {e:?}");
            error_redirect.clone()
        })?;

        let auth_user = AuthUser::from_session(session).await;
        if let Err(ResponseError::NotAuthenticated) = auth_user {
            return Err(error_redirect);
        }

        auth_user.map_err(|e| {
            tracing::error!("{e:?}");
            error_redirect
        })
    }
}

impl OptionalFromRequestParts<AppState> for AuthUser {
    type Rejection = ResponseError;

    async fn from_request_parts(
        req: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Option<Self>, Self::Rejection> {
        let session = Session::from_request_parts(req, state)
            .await
            .map_err(|(_status, description)| anyhow!(description))?;

        let auth_user = AuthUser::from_session(session).await;
        if let Err(ResponseError::NotAuthenticated) = auth_user {
            return Ok(None);
        }

        Ok(Some(auth_user?))
    }
}
