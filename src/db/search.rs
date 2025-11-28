use sqlx::{query, query_as};
use uuid::Uuid;

use super::AppTx;
use crate::response_error::ResponseResult;

pub enum PreviousPage {
    DoesNotExist,
    IsFirstPage,
    AfterBookmarkId(Uuid),
}

pub struct Results {
    pub bookmarks: Vec<Result>,
    pub previous_page: PreviousPage,
    pub next_page_after_bookmark_id: Option<Uuid>,
}

pub struct Result {
    pub title: String,
    pub bookmark_id: Uuid,
    pub bookmark_url: String,
}

pub async fn search(
    tx: &mut AppTx,
    term: &str,
    ap_user_id: Uuid,
    after_bookmark_id: Option<Uuid>,
) -> ResponseResult<Results> {
    let bookmarks = query_as!(
        Result,
        r#"
            select title, url as bookmark_url, id as bookmark_id
            from bookmarks
            where (bookmarks.title ilike '%' || $1 || '%')
                and bookmarks.ap_user_id = $2
                and ($3::uuid is null or bookmarks.id > $3)
            order by bookmarks.id asc
            limit 4
        "#,
        term,
        ap_user_id,
        after_bookmark_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let last_id = bookmarks.last().map(|b| b.bookmark_id);
    let next_page_after_bookmark_id = query!(
        r#"
            select bookmarks.id
            from bookmarks
            where (bookmarks.title ilike '%' || $1 || '%')
                and bookmarks.ap_user_id = $2
                and ($3::uuid is null or bookmarks.id > $3)
        "#,
        term,
        ap_user_id,
        last_id
    )
    .fetch_optional(&mut **tx)
    .await?
    .and(last_id);

    let first_id = bookmarks.first().map(|b| b.bookmark_id);
    // Check if there are *any* bookmarks before the first of the current page.
    // If so, fetch the ids for the previous page and take the first one.
    // We need to fetch multiple bookmarks because we don't know how small the
    // previous page is.
    let previous_bookmarks = query!(
        r#"
            select bookmarks.id
            from bookmarks
            where (bookmarks.title ilike '%' || $1 || '%')
                and bookmarks.ap_user_id = $2
                and ($3::uuid is null or bookmarks.id < $3)
            order by bookmarks.id desc
            limit 5
        "#,
        term,
        ap_user_id,
        first_id
    )
    .fetch_all(&mut **tx)
    .await?;
    let previous_page = if let Some(last) = previous_bookmarks.last() {
        if previous_bookmarks.len() == 5 {
            // There's another page before the previous page, so we can reference the last
            // bookmark of that page.
            PreviousPage::AfterBookmarkId(last.id)
        } else {
            // This is the first page, so we have no bookmark id to query "after"
            PreviousPage::IsFirstPage
        }
    } else {
        // the query returned 0 results, so there is no previous page.
        PreviousPage::DoesNotExist
    };

    Ok(Results {
        bookmarks,
        previous_page,
        next_page_after_bookmark_id,
    })
}
