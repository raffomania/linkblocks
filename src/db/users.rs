use sqlx::{query_as, FromRow};
use uuid::Uuid;

use crate::authentication::hash_password;
use crate::forms::users::{CreateOidcUser, CreateUser};
use crate::response_error::{ResponseError, ResponseResult};

use super::AppTx;

#[derive(FromRow, Debug)]
pub struct User {
  pub id: Uuid,

  // Password-based login data
  #[expect(dead_code)]
  pub username: String,
  pub password_hash: Option<String>,

  // SSO-related data
  #[expect(dead_code)]
  pub email: Option<String>,
  #[expect(dead_code)]
  pub oidc_id: Option<String>,
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

pub async fn insert(tx: &mut AppTx, create_user: CreateUser) -> ResponseResult<User> {
  let hashed_password = hash_password(&create_user.password)?;

  let user = query_as!(
    User,
    r#"
        insert into users
        (username, password_hash)
        values ($1, $2)
        returning *
        "#,
    create_user.username,
    hashed_password,
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

pub async fn create_user_if_not_exists(tx: &mut AppTx, create: CreateUser) -> ResponseResult<User> {
  let username = create.username.clone();
  let user = by_username(tx, &username).await;
  let actual_user = match user {
    Err(ResponseError::NotFound) => {
      tracing::info!("Creating admin user '{username}'");
      insert(tx, create).await?
    }
    Ok(actual_user) => {
      tracing::info!("Admin user '{username}' already exists");
      actual_user
    }
    Err(other) => return Err(other),
  };
  Ok(actual_user)
}
