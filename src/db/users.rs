use sqlx::{query_as, FromRow};
use uuid::Uuid;

use crate::authentication::hash_password;
use crate::forms::users::{CreateUser, CreateOAuthUser};
use crate::response_error::{ResponseError, ResponseResult};

use super::AppTx;

#[derive(FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub using_oauth: bool,
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
}

pub async fn get_all_users(tx: &mut AppTx) -> ResponseResult<Vec<User>> {
    let users = query_as!(
        User,
        r#"
        select * from users
        "#,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(users)
}

pub async fn user_by_oauth_id(tx: &mut AppTx, oauth_id: &str) -> ResponseResult<User> {
    let user = query_as!(
        User,
        r#"
        select * from users
        where oauth_id = $1
        "#,
        oauth_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(user)
}

pub async fn insert_oauth(tx: &mut AppTx, create_user: CreateOAuthUser, oauth_id: &str) -> ResponseResult<User> {
    let hashed_password = hash_password(uuid::Uuid::new_v4().to_string())?;
    let user = query_as!(
        User,
        r#"
        insert into users
        (username, email, oauth_id, oauth_provider, using_oauth, password_hash)
        values ($1, $2, $3, $4, $5, $6)
        returning *"#,
        create_user.username,
        create_user.email,
        create_user.oauth_id,
        "Google",
        true,
        hashed_password
    )
    .fetch_one(&mut **tx)
    .await;
match user 
{
    Ok(user) => {
        println!("User inserted successfully");
        return Ok(user);
    }
    Err(e) => {
        println!("Error inserting user: {:?}", e);
        return Err(e.into());
    }
}

}

pub async fn insert(tx: &mut AppTx, create_user: CreateUser) -> ResponseResult<User> {
    let hashed_password = hash_password(create_user.password)?;

    let user = query_as!(
        User,
        r#"
        insert into users
        (username, password_hash, email)
        values ($1, $2, $3)
        returning *
        "#,
        create_user.username,
        hashed_password,
        Some("")
    ).fetch_one(&mut **tx)
    .await?;
    Ok(user)
}

pub async fn by_id(tx: &mut AppTx, id: Uuid) -> ResponseResult<User> {
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
