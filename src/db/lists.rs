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
    #[allow(dead_code)]
    pub created_at: OffsetDateTime,
    #[allow(dead_code)]
    pub user_id: Uuid,

    pub title: String,
    pub content: Option<String>,
    pub private: bool,
    pub pinned: bool,
}

#[derive(FromRow, Debug, Deserialize, Clone)]
pub struct Metadata {
    pub linked_bookmark_count: i64,
    pub linked_list_count: i64,
    pub user_description: String,
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
        (user_id, title, content, private)
        values ($1, $2, $3, $4)
        returning *"#,
        user_id,
        create_list.title,
        create_list.content,
        create_list.private,
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

pub async fn metadata_by_id(tx: &mut AppTx, list_id: Uuid) -> ResponseResult<Metadata> {
    let list = query_as!(
        Metadata,
        r#"
            select
                coalesce(users.username, users.email) as "user_description!",
                count(links.dest_bookmark_id) as "linked_bookmark_count!",
                count(links.dest_list_id) as "linked_list_count!"
            from lists
            join users on lists.user_id = users.id
            left join links
                on lists.id = links.src_list_id
            where lists.id = $1
            group by users.username, users.email
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
        where user_id = $1 and pinned
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
            limit 500
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lists)
}

pub struct UnpinnedList {
    pub id: Uuid,

    pub title: String,
    pub content: Option<String>,
    pub bookmark_count: i64,
    pub linked_list_count: i64,
}

pub async fn list_unpinned(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<Vec<UnpinnedList>> {
    let lists = query_as!(
        UnpinnedList,
        r#"
            select lists.id, title, content,
                count(links.dest_bookmark_id) as "bookmark_count!",
                count(links.dest_list_id) as "linked_list_count!"
            from lists
            left join links
                on lists.id = links.src_list_id
            where lists.user_id = $1 and not pinned
            group by lists.id
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lists)
}

pub async fn set_private(tx: &mut AppTx, list_id: Uuid, private: bool) -> ResponseResult<List> {
    let list = query_as!(
        List,
        r#"
        update lists
        set private = $1
        where id = $2
        returning *
        "#,
        private,
        list_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(list)
}

pub async fn set_pinned(tx: &mut AppTx, list_id: Uuid, pinned: bool) -> ResponseResult<List> {
    let list = query_as!(
        List,
        r#"
        update lists
        set pinned = $1
        where id = $2
        returning *
        "#,
        pinned,
        list_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(list)
}
