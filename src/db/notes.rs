//! TODO rename notes
//! lists / collections / channels / nodes
use serde::Deserialize;
use sqlx::query_as;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::forms::notes::CreateNote;
use crate::response_error::ResponseResult;

use super::AppTx;
use super::LinkDestination;

#[derive(FromRow, Debug, Deserialize, Clone)]
pub struct Note {
    pub id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub title: String,
    pub content: Option<String>,
}

impl Note {
    pub fn path(&self) -> String {
        let id = self.id;
        format!("/notes/{id}")
    }
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
    create_note: CreateNote,
) -> ResponseResult<Note> {
    let note = query_as!(
        Note,
        r#"
        insert into notes
        (user_id, title, content)
        values ($1, $2, $3)
        returning *"#,
        user_id,
        create_note.title,
        create_note.content,
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

pub async fn search(tx: &mut AppTx, term: &str) -> ResponseResult<Vec<Note>> {
    let notes = query_as!(
        Note,
        r#"
            select *
            from notes
            where notes.title ilike '%' || $1 || '%'
            limit 10
        "#,
        term
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(notes)
}

pub async fn list_recent(tx: &mut AppTx) -> ResponseResult<Vec<Note>> {
    let notes = query_as!(
        Note,
        r#"
            select notes.*
            from notes
            left join links as src_links on notes.id = src_links.src_note_id
            left join links as dest_links on notes.id = dest_links.dest_note_id
            group by notes.id
            order by
                max(src_links.created_at) desc nulls last,
                max(dest_links.created_at) nulls last,
                max(notes.created_at) desc
            limit 10
        "#,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(notes)
}
