use serde::Deserialize;
use sqlx::{query, query_as, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    db,
    response_error::ResponseResult,
    schemas::links::{CreateLink, ReferenceType},
};

use super::AppTx;

#[derive(FromRow)]
pub struct Link {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub src_bookmark_id: Option<Uuid>,
    pub src_note_id: Option<Uuid>,
    pub src_list_id: Option<Uuid>,

    pub dest_bookmark_id: Option<Uuid>,
    pub dest_note_id: Option<Uuid>,
    pub dest_list_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum LinkDestinationWithChildren {
    Bookmark(db::Bookmark),
    Note(db::NoteWithLinks),
    List(db::List),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LinkDestination {
    Bookmark(db::Bookmark),
    Note(db::Note),
    List(db::List),
}

pub struct LinkWithContent {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub dest: LinkDestinationWithChildren,
}

pub async fn insert(tx: &mut AppTx, user_id: Uuid, create: CreateLink) -> ResponseResult<Link> {
    let src_bookmark_id = (create.src_ref_type == ReferenceType::Bookmark).then_some(create.src_id);
    let src_note_id = (create.src_ref_type == ReferenceType::Note).then_some(create.src_id);
    let src_list_id = (create.src_ref_type == ReferenceType::List).then_some(create.src_id);

    let dest_bookmark_id =
        (create.dest_ref_type == ReferenceType::Bookmark).then_some(create.dest_id);
    let dest_note_id = (create.dest_ref_type == ReferenceType::Note).then_some(create.dest_id);
    let dest_list_id = (create.dest_ref_type == ReferenceType::List).then_some(create.dest_id);

    let list = query_as!(
        Link,
        r#"
        insert into links
        (
            user_id,
            src_bookmark_id,
            src_note_id,
            src_list_id,
            dest_bookmark_id,
            dest_note_id,
            dest_list_id
        )
        values ($1, $2, $3, $4, $5, $6, $7)
        returning *"#,
        user_id,
        src_bookmark_id,
        src_note_id,
        src_list_id,
        dest_bookmark_id,
        dest_note_id,
        dest_list_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(list)
}

pub async fn list_by_list(tx: &mut AppTx, list_id: Uuid) -> ResponseResult<Vec<LinkWithContent>> {
    let rows = query!(
        r#"
        select
            links.id as link_id,
            links.created_at as link_created_at,
            links.user_id as link_user_id,

            case when notes.id is not null then
                json_object(
                    'note': to_json(notes.*),
                    'links':
                        jsonb_agg(notes_bookmarks.*) filter (where notes_bookmarks.id is not null)
                        || jsonb_agg(notes_notes.*) filter (where notes_notes.id is not null)
                        || jsonb_agg(notes_lists.*) filter (where notes_lists.id is not null)
                )
            when bookmarks.id is not null then
                to_jsonb(bookmarks.*)
            when lists.id is not null then
                to_jsonb(lists.*)
            else null end as dest
        from links

        left join notes on notes.id = links.dest_note_id
        left join links as notes_links on notes_links.src_note_id = notes.id
        left join bookmarks as notes_bookmarks on notes_bookmarks.id = notes_links.dest_bookmark_id
        left join notes as notes_notes on notes_notes.id = notes_links.dest_note_id
        left join lists as notes_lists on notes_lists.id = notes_links.dest_list_id

        left join bookmarks on bookmarks.id = links.dest_bookmark_id

        left join lists on lists.id = links.dest_list_id

        where links.src_list_id = $1
        group by links.id, notes.id, bookmarks.id, lists.id
        -- temporary hack for random ordering of demo data
        order by link_id
        "#,
        list_id
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
