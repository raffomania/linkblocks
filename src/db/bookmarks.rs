use serde::Deserialize;
use sqlx::{query_as, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::response_error::ResponseResult;

use super::AppTx;

#[derive(FromRow, Debug, Deserialize, Clone)]
pub struct Bookmark {
    pub id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    #[expect(dead_code)]
    pub created_at: OffsetDateTime,
    #[expect(dead_code)]
    pub user_id: Uuid,

    pub url: String,
    pub title: String,
}

impl Bookmark {
    pub fn path(&self) -> String {
        let id = self.id;
        format!("/bookmarks/{id}")
    }
}

pub struct InsertBookmark {
    pub url: String,
    pub title: String,
}

pub async fn insert(
    tx: &mut AppTx,
    user_id: Uuid,
    create_bookmark: InsertBookmark,
) -> ResponseResult<Bookmark> {
    let bookmark = query_as!(
        Bookmark,
        r#"
        insert into bookmarks
        (user_id, url, title)
        values ($1, $2, $3)
        returning *"#,
        user_id,
        create_bookmark.url,
        create_bookmark.title
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(bookmark)
}

pub async fn list_unsorted(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<Vec<Bookmark>> {
    let bookmarks = query_as!(
        Bookmark,
        r#"
        select *
        from bookmarks
        where user_id = $1
        and not exists (
            select null from links
            where dest_bookmark_id = bookmarks.id
        );
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(bookmarks)
}

pub async fn delete_by_id(tx: &mut AppTx, id: Uuid) -> ResponseResult<Bookmark> {
    let bookmark = query_as!(
        Bookmark,
        r#"
        delete from bookmarks
        where id = $1
        returning *;
        "#,
        id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(bookmark)
}
