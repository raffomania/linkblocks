use activitypub_federation::fetch::object_id::ObjectId;
use serde::Deserialize;
use sqlx::{FromRow, query_as};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use super::AppTx;
use crate::response_error::ResponseResult;

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

    pub ap_id: ObjectId<Bookmark>,
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
        Bookmark,
        r#"
        insert into bookmarks
        (id, user_id, url, title, ap_id)
        values ($1, $2, $3, $4, $5)
        returning *"#,
        id,
        user_id,
        create_bookmark.url,
        create_bookmark.title,
        ap_id
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

pub async fn upsert(tx: &mut AppTx, create_user: CreateApUser) -> ResponseResult<ApUser> {
    let user = query_as!(
        ApUserRow,
        r#"
        insert into bookmarks
        (user_id, url, title)
        values ($1, $2, $3)
        on conflict(ap_id) do update set
            ap_id = $1,
            username = $2,
            inbox_url = $3,
            public_key = $4,
            private_key = $5,
            last_refreshed_at = $6,
            display_name = $7,
            bio = $8
        returning *
        "#,
        create_user.ap_id.to_string(),
        create_user.username,
        create_user.inbox_url.to_string(),
        create_user.public_key,
        create_user.private_key,
        create_user.last_refreshed_at,
        create_user.display_name,
        create_user.bio,
    )
    .fetch_one(&mut **tx)
    .await?
    .try_into()?;

    Ok(user)
}
