use sqlx::{query_as, FromRow};
use uuid::Uuid;

use crate::app_error::{AppError, AppResult};
use crate::authentication::hash_password;
use crate::schemas::users::CreateUser;

use super::AppTx;

#[derive(FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

pub async fn insert(tx: &mut AppTx, create: CreateUser) -> AppResult<User> {
    let hashed_password = hash_password(create.password)?;

    let user = query_as!(
        User,
        r#"
        insert into users
        (password_hash, username)
        values ($1, $2)
        returning *"#,
        hashed_password,
        create.username
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(user)
}

pub async fn by_id(tx: &mut AppTx, id: Uuid) -> AppResult<User> {
    let user = query_as!(
        User,
        r#"
        select * from users
        where id = $1
    "#,
        id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(user)
}
pub async fn by_username(tx: &mut AppTx, username: &str) -> AppResult<User> {
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

pub async fn create_user_if_not_exists(tx: &mut AppTx, create: CreateUser) -> AppResult<User> {
    let username = create.username.clone();
    let user = by_username(tx, &username).await;
    let actual_user = match user {
        Err(AppError::NotFound) => {
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
