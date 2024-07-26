use serde::Serialize;
use sqlx::{query_as, FromRow};
use time;
use uuid::Uuid;

use crate::response_error::ResponseResult;

use super::AppTx;

#[derive(FromRow, Debug, Serialize)]
pub struct UserApi {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key: Option<String>,
    pub permissions: Option<String>,
    pub valid_until: time::OffsetDateTime,
    pub created_at: time::OffsetDateTime,
}

pub async fn insert(
    tx: &mut AppTx,
    user_id: Uuid,
    api_key: Uuid,
    valid_until: time::OffsetDateTime,
    permissions: &str,
) -> ResponseResult<UserApi> {
    let user_api = query_as!(
        UserApi,
        r#"
        insert into user_apis
        (user_id, api_key, permissions, valid_until)
        values ($1, $2, $3, $4)
        returning *"#,
        user_id,
        Some(api_key.to_string()),
        Some(permissions),
        valid_until
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(user_api)
}

pub async fn renew(
    tx: &mut AppTx,
    user_id: Uuid,
    api_key: Uuid,
) -> ResponseResult<UserApi> {
    let user_api = query_as!(
        UserApi,
        r#"
        update user_apis
        set api_key = $1
        where user_id = $2
        returning *"#,
        api_key.to_string(),
        user_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(user_api)
}

pub async fn by_api_key(tx: &mut AppTx, api_key: &str) -> ResponseResult<UserApi> {
    let user_api = query_as!(
        UserApi,
        r#"
        select * from user_apis
        where api_key = $1
        "#,
        api_key
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(user_api)
}

pub async fn by_user_id(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<UserApi> {
    let user_api = query_as!(
        UserApi,
        r#"
        select * from user_apis
        where user_id = $1
        "#,
        user_id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(user_api)
}
