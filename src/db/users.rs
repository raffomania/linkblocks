use sqlx::{FromRow, query_as};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use super::AppTx;
use crate::{
    authentication::hash_password,
    federation,
    forms::{
        ap_users::CreateApUser,
        users::{CreateOidcUser, CreateUser},
    },
    response_error::{ResponseError, ResponseResult},
};

#[derive(FromRow, Debug)]
pub struct User {
    pub id: Uuid,

    // TODO this is only used in tests so far, which breaks
    // `#[expect(dead_code)]` for some reason
    #[allow(dead_code)]
    pub username: String,

    // Password-based login data
    pub password_hash: Option<String>,

    // SSO-related data
    #[expect(dead_code)]
    pub email: Option<String>,
    #[expect(dead_code)]
    pub oidc_id: Option<String>,

    // ActivityPub data
    #[expect(dead_code)]
    pub ap_user_id: Option<Uuid>,
}

pub async fn user_by_oidc_id(tx: &mut AppTx, oidc_id: &str) -> ResponseResult<User> {
    let user = query_as!(
        User,
        r#"
        select * from users
        where oidc_id = $1
        "#,
        oidc_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(user)
}

pub async fn insert_oidc(tx: &mut AppTx, create_user: CreateOidcUser) -> ResponseResult<User> {
    let user = query_as!(
        User,
        r#"
        insert into users
        (email, oidc_id, username)
        values ($1, $2, $3)
        returning *"#,
        create_user.email,
        create_user.oidc_id,
        create_user.username
    )
    .fetch_one(&mut **tx)
    .await;
    match user {
        Ok(user) => Ok(user),
        Err(e) => {
            tracing::warn!("Error inserting user: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn insert(
    tx: &mut AppTx,
    create_user: CreateUser,
    base_url: &Url,
) -> ResponseResult<User> {
    let hashed_password = hash_password(&create_user.password)?;
    let ap_keypair = federation::signing::generate_keypair()?;

    let ap_id = base_url.join("/ap/user/")?.join(&create_user.username)?;
    let inbox_url = base_url.join("/ap/inbox")?;

    let create_ap_user = CreateApUser {
        ap_id,
        username: create_user.username.clone(),
        inbox_url,
        public_key: ap_keypair.public_key,
        private_key: Some(ap_keypair.private_key),
        last_refreshed_at: OffsetDateTime::now_utc(),
        display_name: None,
        bio: None,
    };
    let ap_user = super::ap_users::insert(tx, create_ap_user).await?;

    let user = query_as!(
        User,
        r#"
        insert into users
        (username, password_hash, ap_user_id)
        values ($1, $2, $3)
        returning *
        "#,
        create_user.username,
        hashed_password,
        ap_user.id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(user)
}

pub async fn by_username(tx: &mut AppTx, username: &str) -> ResponseResult<User> {
    let user = query_as!(
        User,
        r#"
        select * from users
        where username = $1
        "#,
        username
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(user)
}

pub async fn create_user_if_not_exists(
    tx: &mut AppTx,
    create: CreateUser,
    base_url: &Url,
) -> ResponseResult<User> {
    let username = create.username.clone();
    let user = by_username(tx, &username).await;
    let actual_user = match user {
        Err(ResponseError::NotFound) => {
            tracing::info!("Creating admin user '{username}'");
            insert(tx, create, base_url).await?
        }
        Ok(actual_user) => {
            tracing::info!("Admin user '{username}' already exists");
            actual_user
        }
        Err(other) => return Err(other),
    };
    Ok(actual_user)
}
