use anyhow::anyhow;
use sqlx::prelude::FromRow;
use sqlx::{query, query_as, Postgres, Transaction};
use uuid::Uuid;

use crate::app_error::{AppError, Result};
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

pub async fn by_username(db: &mut Transaction<'_, Postgres>, username: String) -> Result<User> {
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
    tracing::info!("Checking if admin user '{username}' exists...");
    let user = by_username(tx, username).await;
    match user {
        Err(AppError::NotFound()) => {
            tracing::info!("Creating admin user");
            insert(tx, create).await?;
        }
        Ok(_) => {
            tracing::info!("User already exists")
        }
        Err(other) => return Err(other),
    }
    Ok(())
}

fn hash_password(password: String) -> Result<String> {
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    Ok(
        argon2::PasswordHasher::hash_password(&argon2, password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {e}"))?
            .to_string(),
    )
}
