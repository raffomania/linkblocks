use serde::Deserialize;
use sqlx::{query, query_as, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{db, forms::links::CreateLink, response_error::ResponseResult};

use super::AppTx;

#[derive(FromRow, Debug)]
#[allow(dead_code)]
pub struct Link {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub src_list_id: Option<Uuid>,

    pub dest_bookmark_id: Option<Uuid>,
    pub dest_list_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum LinkDestinationWithChildren {
    Bookmark(db::Bookmark),
    List(db::ListWithLinks),
}

impl LinkDestinationWithChildren {
    pub fn id(&self) -> Uuid {
        match self {
            LinkDestinationWithChildren::Bookmark(b) => b.id,
            LinkDestinationWithChildren::List(l) => l.list.id,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum LinkDestination {
    Bookmark(db::Bookmark),
    List(db::List),
}

impl LinkDestination {
    pub fn id(&self) -> Uuid {
        match self {
            LinkDestination::Bookmark(b) => b.id,
            LinkDestination::List(n) => n.id,
        }
    }

    pub fn path(&self) -> String {
        match self {
            LinkDestination::Bookmark(b) => b.path(),
            LinkDestination::List(n) => n.path(),
        }
    }
}

pub struct LinkWithContent {
    pub id: Uuid,
    #[allow(dead_code)]
    pub created_at: OffsetDateTime,
    #[allow(dead_code)]
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
            src_list_id,
            dest_bookmark_id,
            dest_list_id
        )
        values ($1,
            (select id from lists where id = $2),
            (select id from bookmarks where id = $3),
            (select id from lists where id = $3)
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

pub async fn list_by_list(tx: &mut AppTx, list_id: Uuid) -> ResponseResult<Vec<LinkWithContent>> {
    let rows = query!(
        r#"
        select
            links.id as link_id,
            links.created_at as link_created_at,
            links.user_id as link_user_id,

            case when lists.id is not null then
                jsonb_build_object(
                    'list', to_jsonb(lists.*),
                    'links',
                    coalesce(
                        jsonb_agg(lists_bookmarks.*)
                        filter (where lists_bookmarks.id is not null),
                    jsonb_build_array())
                    || coalesce(
                        jsonb_agg(lists_lists.*)
                        filter (where lists_lists.id is not null),
                    jsonb_build_array())
                )
            when bookmarks.id is not null then
                to_jsonb(bookmarks.*)
            else null end as dest
        from links

        left join lists on lists.id = links.dest_list_id
        left join links as lists_links on lists_links.src_list_id = lists.id
        left join bookmarks as lists_bookmarks on lists_bookmarks.id = lists_links.dest_bookmark_id
        left join lists as lists_lists on lists_lists.id = lists_links.dest_list_id

        left join bookmarks on bookmarks.id = links.dest_bookmark_id

        where links.src_list_id = $1
        group by links.id, lists.id, bookmarks.id
        order by links.created_at desc
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

pub async fn delete_by_id(tx: &mut AppTx, id: Uuid) -> ResponseResult<Link> {
    let link = query_as!(
        Link,
        r#"
        delete from links
        where id = $1
        returning *
        "#,
        id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(link)
}
