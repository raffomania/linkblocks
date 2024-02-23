use serde::Deserialize;
use sqlx::{query_as, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db;
use crate::forms::bookmarks::CreateBookmark;
use crate::forms::links::CreateLink;
use crate::response_error::ResponseResult;

use super::AppTx;

#[derive(FromRow, Debug, Deserialize, Clone)]
pub struct Bookmark {
    pub id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
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

pub async fn insert(
    tx: &mut AppTx,
    user_id: Uuid,
    create_bookmark: CreateBookmark,
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

    if let Some(parent) = create_bookmark.parent {
        db::links::insert(
            tx,
            user_id,
            CreateLink {
                src: parent,
                dest: bookmark.id,
            },
        )
        .await?;
    }

    Ok(bookmark)
}
