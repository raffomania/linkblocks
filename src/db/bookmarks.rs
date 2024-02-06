use sqlx::{query_as, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app_error::AppResult;
use crate::schemas::bookmarks::CreateBookmark;

use super::AppTx;

#[derive(FromRow, Debug)]
pub struct Bookmark {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub url: String,
    pub title: String,
}

pub async fn insert(tx: &mut AppTx, user_id: Uuid, create: CreateBookmark) -> AppResult<Bookmark> {
    let bookmark = query_as!(
        Bookmark,
        r#"
        insert into bookmarks
        (user_id, url, title)
        values ($1, $2, $3)
        returning *"#,
        user_id,
        create.url,
        create.title
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(bookmark)
}
