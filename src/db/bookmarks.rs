use serde::Deserialize;
use sqlx::{FromRow, query_as};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use super::AppTx;
use crate::response_error::{ResponseError, ResponseResult};

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
    // TODO change this once we implement ap::Object
    // pub ap_id: ObjectId<Bookmark>,
    #[expect(dead_code)]
    pub ap_id: String,
}

#[derive(FromRow, Debug)]
struct BookmarkRow {
    id: Uuid,
    created_at: OffsetDateTime,
    user_id: Uuid,

    url: String,
    title: String,
    ap_id: String,
}

impl TryFrom<BookmarkRow> for Bookmark {
    type Error = ResponseError;

    fn try_from(value: BookmarkRow) -> Result<Self, Self::Error> {
        Ok(Bookmark {
            id: value.id,
            created_at: value.created_at,
            user_id: value.user_id,
            url: value.url,
            title: value.title,
            // ap_id: value.ap_id.parse()?,
            ap_id: value.ap_id,
        })
    }
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
    base_url: &Url,
) -> ResponseResult<Bookmark> {
    let id = Uuid::new_v4();
    let ap_id = base_url.join("/ap/bookmark/")?.join(&id.to_string())?;
    let bookmark = query_as!(
        BookmarkRow,
        r#"
        insert into bookmarks
        (id, user_id, url, title, ap_id)
        values ($1, $2, $3, $4, $5)
        returning *"#,
        id,
        user_id,
        create_bookmark.url,
        create_bookmark.title,
        ap_id.to_string()
    )
    .fetch_one(&mut **tx)
    .await?;

    bookmark.try_into()
}

pub async fn list_unsorted(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<Vec<Bookmark>> {
    let bookmarks = query_as!(
        BookmarkRow,
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

    bookmarks
        .into_iter()
        .map(Bookmark::try_from)
        .collect::<ResponseResult<Vec<_>>>()
}

pub async fn delete_by_id(tx: &mut AppTx, id: Uuid) -> ResponseResult<Bookmark> {
    let bookmark = query_as!(
        BookmarkRow,
        r#"
        delete from bookmarks
        where id = $1
        returning *;
        "#,
        id
    )
    .fetch_one(&mut **tx)
    .await?;

    bookmark.try_into()
}
