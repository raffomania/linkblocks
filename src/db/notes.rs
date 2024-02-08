use serde::Deserialize;
use sqlx::query_as;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::response_error::ResponseResult;
use crate::schemas::notes::CreateNote;

use super::AppTx;
use super::LinkDestination;

#[derive(FromRow, Debug, Deserialize)]
pub struct Note {
    pub id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub title: String,
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteWithLinks {
    pub note: Note,

    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_default_from_null")]
    pub links: Vec<LinkDestination>,
}

pub async fn insert(
    tx: &mut AppTx,
    user_id: Uuid,
    // TODO rename to create_<entity> everywhere
    create: CreateNote,
) -> ResponseResult<Note> {
    let note = query_as!(
        Note,
        r#"
        insert into notes
        (user_id, title, content)
        values ($1, $2, $3)
        returning *"#,
        user_id,
        create.title,
        create.content,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(note)
}

pub async fn by_id(tx: &mut AppTx, note_id: Uuid) -> ResponseResult<Note> {
    let note = query_as!(
        Note,
        r#"
        select * from notes
        where id = $1
        "#,
        note_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(note)
}

pub async fn list_pinned_by_user(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<Vec<Note>> {
    let notes = query_as!(
        Note,
        r#"
        select * from notes
        where user_id = $1
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(notes)
}
