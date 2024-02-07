use serde::Deserialize;
use sqlx::{query_as, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::response_error::ResponseResult;
use crate::schemas::lists::CreateList;

use super::AppTx;

#[derive(FromRow, Debug, Deserialize)]
pub struct List {
    pub id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub title: String,
}

pub async fn insert(tx: &mut AppTx, user_id: Uuid, create: CreateList) -> ResponseResult<List> {
    let list = query_as!(
        List,
        r#"
        insert into lists
        (user_id, title)
        values ($1, $2)
        returning *"#,
        user_id,
        create.title
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(list)
}

pub async fn list_by_user(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<Vec<List>> {
    Ok(query_as!(
        List,
        r#"
        select * from lists
        where user_id = $1
        "#,
        user_id
    )
    .fetch_all(&mut **tx)
    .await?)
}

pub async fn by_id(tx: &mut AppTx, list_id: Uuid) -> ResponseResult<List> {
    Ok(query_as!(
        List,
        r#"
        select * from lists
        where id = $1
        "#,
        list_id
    )
    .fetch_one(&mut **tx)
    .await?)
}
