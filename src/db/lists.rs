use serde::Deserialize;
use sqlx::query_as;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::forms::lists::CreateList;
use crate::response_error::ResponseResult;

use super::AppTx;
use super::LinkDestination;

#[derive(FromRow, Debug, Deserialize, Clone)]
pub struct List {
    pub id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub title: String,
    pub content: Option<String>,
}

impl List {
    pub fn path(&self) -> String {
        let id = self.id;
        format!("/lists/{id}")
    }
}

#[derive(Deserialize)]
pub struct ListWithLinks {
    pub list: List,

    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_default_from_null")]
    pub links: Vec<LinkDestination>,
}

pub async fn insert(
    tx: &mut AppTx,
    user_id: Uuid,
    create_list: CreateList,
) -> ResponseResult<List> {
    let list = query_as!(
        List,
        r#"
        insert into lists
        (user_id, title, content)
        values ($1, $2, $3)
        returning *"#,
        user_id,
        create_list.title,
        create_list.content,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(list)
}

pub async fn by_id(tx: &mut AppTx, list_id: Uuid) -> ResponseResult<List> {
    let list = query_as!(
        List,
        r#"
        select * from lists
        where id = $1
        "#,
        list_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(list)
}

pub async fn list_by_id(tx: &mut AppTx, list_ids: &[Uuid]) -> ResponseResult<Vec<List>> {
    let list = query_as!(
        List,
        r#"
        select * from lists
        where id = any($1)
        "#,
        list_ids,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(list)
}

pub async fn list_pinned_by_user(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<Vec<List>> {
    let lists = query_as!(
        List,
        r#"
        select * from lists
        where user_id = $1
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lists)
}

pub async fn search(tx: &mut AppTx, term: &str, user_id: Uuid) -> ResponseResult<Vec<List>> {
    let lists = query_as!(
        List,
        r#"
            select *
            from lists
            where (lists.title ilike '%' || $1 || '%')
            and lists.user_id = $2
            limit 10
        "#,
        term,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lists)
}

pub async fn list_recent(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<Vec<List>> {
    let lists = query_as!(
        List,
        r#"
            select lists.*
            from lists
            left join links as src_links on lists.id = src_links.src_list_id
            left join links as dest_links on lists.id = dest_links.dest_list_id
            where lists.user_id = $1
            group by lists.id
            order by
                max(src_links.created_at) desc nulls last,
                max(dest_links.created_at) nulls last,
                max(lists.created_at) desc
            limit 10
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lists)
}
