use sqlx::{query_as, FromRow, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app_error::AppResult;
use crate::schemas::lists::CreateList;

#[derive(FromRow, Debug)]
pub struct List {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub title: String,
}

pub async fn insert(
    db: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    create: CreateList,
) -> AppResult<List> {
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
    .fetch_one(&mut **db)
    .await?;

    Ok(list)
}
