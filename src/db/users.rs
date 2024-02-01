use sqlx::prelude::FromRow;
use sqlx::{query, query_as, Postgres, Transaction};
use uuid::Uuid;

use crate::app_error::{AppError, Result};
use crate::authentication::hash_password;
use crate::schemas::users::CreateUser;

#[derive(FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

pub async fn insert(db: &mut Transaction<'_, Postgres>, create: CreateUser) -> Result<()> {
    let hashed_password = hash_password(create.password)?;

    query!(
        r#"
        insert into users 
        (password_hash, username) 
        values ($1, $2)"#,
        hashed_password,
        create.username
    )
    .execute(&mut **db)
    .await?;

    Ok(())
}

pub async fn by_id(db: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<User> {
    let user = query_as!(
        User,
        r#"
        select * from users
        where id = $1
    "#,
        id
    )
    .fetch_one(&mut **db)
    .await?;

    Ok(user)
}
pub async fn by_username(db: &mut Transaction<'_, Postgres>, username: &str) -> Result<User> {
    let user = query_as!(
        User,
        r#"
        select * from users
        where username = $1
    "#,
        username
    )
    .fetch_one(&mut **db)
    .await?;

    Ok(user)
}

pub async fn create_user_if_not_exists(
    tx: &mut Transaction<'_, Postgres>,
    create: CreateUser,
) -> Result<()> {
    let username = create.username.clone();
    let user = by_username(tx, &username).await;
    match user {
        Err(AppError::NotFound) => {
            tracing::info!("Creating admin user '{username}'");
            insert(tx, create).await?;
        }
        Ok(_) => {
            tracing::info!("Admin user '{username}' already exists")
        }
        Err(other) => return Err(other),
    }
    Ok(())
}
