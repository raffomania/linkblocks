use sqlx::{query_as, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app_error::AppResult;
use crate::schemas::lists::CreateList;

use super::AppTx;

#[derive(FromRow, Debug)]
pub struct List {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub title: String,
}

pub async fn insert(tx: &mut AppTx, user_id: Uuid, create: CreateList) -> AppResult<List> {
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

pub async fn list_by_user(tx: &mut AppTx, user_id: Uuid) -> AppResult<Vec<List>> {
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

pub async fn by_id(tx: &mut AppTx, list_id: Uuid) -> AppResult<List> {
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
