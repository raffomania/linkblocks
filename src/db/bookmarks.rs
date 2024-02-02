use sqlx::{query_as, FromRow, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app_error::AppResult;
use crate::schemas::bookmarks::CreateBookmark;

#[derive(FromRow, Debug)]
pub struct Bookmark {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub url: String,
}

pub async fn insert(
    db: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    create: CreateBookmark,
) -> AppResult<Bookmark> {
    let bookmark = query_as!(
        Bookmark,
        r#"
        insert into bookmarks 
        (user_id, url) 
        values ($1, $2)
        returning *"#,
        user_id,
        create.url
    )
    .fetch_one(&mut **db)
    .await?;

    Ok(bookmark)
}
