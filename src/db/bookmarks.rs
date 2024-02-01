use crate::app_error::Result;
use sqlx::{query, FromRow, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::schemas::bookmarks::CreateBookmark;

#[derive(FromRow, Debug)]
pub struct Bookmark {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub url: String,
}

pub async fn insert(db: &mut Transaction<'_, Postgres>, create: CreateBookmark) -> Result<()> {
    query!(
        r#"
        insert into bookmarks 
        (user_id, url) 
        values ($1, $2)"#,
        create.user_id,
        create.url
    )
    .execute(&mut **db)
    .await?;

    Ok(())
}
