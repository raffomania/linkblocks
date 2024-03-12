//! todo: better name than "items"
//! maybe LinkDestinations?

use anyhow::Context;
use sqlx::query;
use uuid::Uuid;

use crate::response_error::ResponseResult;

use super::{AppTx, LinkDestination};

pub async fn search(
    tx: &mut AppTx,
    term: &str,
    user_id: Uuid,
) -> ResponseResult<Vec<LinkDestination>> {
    let jsons = query!(
        r#"
            select to_jsonb(bookmarks.*) as item
            from bookmarks
            where bookmarks.title ilike '%' || $1 || '%'
            and bookmarks.user_id = $2
            union
            select to_jsonb(lists.*) as item
            from lists
            where lists.title ilike '%' || $1 || '%'
            and lists.user_id = $2
            limit 10
        "#,
        term,
        user_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let results = jsons
        .into_iter()
        .map(|row| Ok(serde_json::from_value(row.item.into())?))
        .collect::<anyhow::Result<Vec<LinkDestination>>>()?;

    Ok(results)
}

pub async fn list_recent(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<Vec<LinkDestination>> {
    // TODO order by max(links.created_at, lists.created_at, bookmarks.created_at)
    let jsons = query!(
        r#"
            select
            case
                when lists.id is not null then
                    to_jsonb(lists.*)
                when bookmarks.id is not null then
                    to_jsonb(bookmarks.*)
                else null
            end as item
            from links
            left join lists
                on lists.id = links.dest_list_id
            left join bookmarks
                on bookmarks.id = links.dest_bookmark_id
            where
                (lists.id is not null or bookmarks.id is not null)
                and links.user_id = $1
            order by
                links.created_at desc nulls last,
                lists.created_at desc,
                bookmarks.created_at desc
            limit 10
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    let results = jsons
        .into_iter()
        .map(|row| Ok(serde_json::from_value(row.item.into())?))
        .collect::<anyhow::Result<Vec<LinkDestination>>>()?;

    Ok(results)
}

pub async fn by_id(tx: &mut AppTx, id: Uuid) -> ResponseResult<LinkDestination> {
    let json = query!(
        r#"
            select to_jsonb(bookmarks.*) as item
            from bookmarks
            where bookmarks.id = $1
            union
            select to_jsonb(lists.*) as item
            from lists
            where lists.id = $1
        "#,
        id
    )
    .fetch_one(&mut **tx)
    .await?;

    let results =
        serde_json::from_value(json.item.into()).context("Failed to deserialize item from DB")?;

    Ok(results)
}
