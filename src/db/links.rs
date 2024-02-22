use serde::Deserialize;
use sqlx::{query, query_as, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{db, forms::links::CreateLink, response_error::ResponseResult};

use super::AppTx;

#[derive(FromRow)]
pub struct Link {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub src_bookmark_id: Option<Uuid>,
    pub src_note_id: Option<Uuid>,

    pub dest_bookmark_id: Option<Uuid>,
    pub dest_note_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum LinkDestinationWithChildren {
    Bookmark(db::Bookmark),
    Note(db::NoteWithLinks),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum LinkDestination {
    Bookmark(db::Bookmark),
    Note(db::Note),
}

impl LinkDestination {
    pub fn id(&self) -> Uuid {
        match self {
            LinkDestination::Bookmark(b) => b.id,
            LinkDestination::Note(n) => n.id,
        }
    }

    pub fn path(&self) -> String {
        let prefix = match self {
            LinkDestination::Bookmark(_) => "bookmarks",
            LinkDestination::Note(_) => "notes",
        };
        let id = self.id();
        format!("/{prefix}/{id}")
    }
}

pub struct LinkWithContent {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub dest: LinkDestinationWithChildren,
}

pub async fn insert(
    tx: &mut AppTx,
    user_id: Uuid,
    create_link: CreateLink,
) -> ResponseResult<Link> {
    let list = query_as!(
        Link,
        r#"
        insert into links
        (
            user_id,
            src_bookmark_id,
            src_note_id,
            dest_bookmark_id,
            dest_note_id
        )
        values ($1,
            (select id from bookmarks where id = $2),
            (select id from notes where id = $2),
            (select id from bookmarks where id = $3),
            (select id from notes where id = $3)
        )
        returning *"#,
        user_id,
        create_link.src,
        create_link.dest
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(list)
}

pub async fn list_by_note(tx: &mut AppTx, note_id: Uuid) -> ResponseResult<Vec<LinkWithContent>> {
    let rows = query!(
        r#"
        select
            links.id as link_id,
            links.created_at as link_created_at,
            links.user_id as link_user_id,

            case when notes.id is not null then
                json_object(
                    'note': to_jsonb(notes.*),
                    'links': jsonb_agg_strict(notes_bookmarks.*)
                        || jsonb_agg_strict(notes_notes.*)
                )
            when bookmarks.id is not null then
                to_jsonb(bookmarks.*)
            else null end as dest
        from links

        left join notes on notes.id = links.dest_note_id
        left join links as notes_links on notes_links.src_note_id = notes.id
        left join bookmarks as notes_bookmarks on notes_bookmarks.id = notes_links.dest_bookmark_id
        left join notes as notes_notes on notes_notes.id = notes_links.dest_note_id

        left join bookmarks on bookmarks.id = links.dest_bookmark_id

        where links.src_note_id = $1
        group by links.id, notes.id, bookmarks.id
        -- temporary hack for random ordering of demo data
        order by link_id
        "#,
        note_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let results = rows
        .into_iter()
        .map(|row| {
            let dest: LinkDestinationWithChildren = serde_json::from_value(row.dest.into())?;
            Ok(LinkWithContent {
                id: row.link_id,
                created_at: row.link_created_at,
                user_id: row.link_user_id,
                dest,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(results)
}
