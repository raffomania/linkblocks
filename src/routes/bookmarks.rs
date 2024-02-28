use anyhow::Context;
use askama_axum::IntoResponse;
use axum::{
    extract::{Form, Path, Query},
    http::HeaderMap,
    response::{Redirect, Response},
    routing::{delete, get},
    Router,
};

use serde::Deserialize;
use uuid::Uuid;

use crate::{
    authentication::AuthUser,
    db::{self, bookmarks::InsertBookmark},
    extract,
    forms::bookmarks::CreateBookmark,
    response_error::ResponseResult,
    server::AppState,
    views::{
        self,
        bookmarks::{CreateBookmarkTemplate, UnlinkedBookmarksTemplate},
        layout::LayoutTemplate,
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bookmarks/create", get(get_create).post(post_create))
        .route("/bookmarks/unlinked", get(get_unlinked))
        .route("/bookmarks/:id", delete(delete_by_id))
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Form(input): Form<CreateBookmark>,
) -> ResponseResult<Response> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    let selected_parent = match input.parent {
        Some(id) => Some(db::notes::by_id(&mut tx, id).await?),
        None => None,
    };

    let search_results = match (input.note_search_term.as_ref(), input.parent) {
        (None, None) => db::notes::list_recent(&mut tx).await?,
        (Some(term), None) => db::notes::search(&mut tx, term).await?,
        _ => Vec::new(),
    };

    let insert_bookmark = match InsertBookmark::try_from(input.clone()) {
        Err(errors) => {
            return Ok(views::bookmarks::CreateBookmarkTemplate {
                layout,
                errors,
                input,
                selected_parent,
                search_results,
            }
            .into_response());
        }
        Ok(i) => i,
    };

    db::bookmarks::insert(&mut tx, auth_user.user_id, insert_bookmark).await?;

    tx.commit().await?;

    let redirect_dest = match selected_parent {
        Some(parent) => parent.path(),
        None => "/bookmarks/unlinked".to_string(),
    };
    Ok(Redirect::to(&redirect_dest).into_response())
}

#[derive(Deserialize)]
struct CreateBookmarkQuery {
    parent_id: Option<Uuid>,
    url: Option<String>,
    title: Option<String>,
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Query(query): Query<CreateBookmarkQuery>,
) -> ResponseResult<CreateBookmarkTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    let selected_parent = match query.parent_id {
        Some(id) => Some(db::notes::by_id(&mut tx, id).await?),
        _ => None,
    };

    Ok(CreateBookmarkTemplate {
        layout,
        errors: Default::default(),
        input: CreateBookmark {
            parent: selected_parent.as_ref().map(|p| p.id),
            url: query.url.unwrap_or_default(),
            title: query.title.unwrap_or_default(),
            ..Default::default()
        },
        selected_parent,
        search_results: db::notes::list_recent(&mut tx).await?,
    })
}

async fn get_unlinked(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<UnlinkedBookmarksTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;
    let bookmarks = db::bookmarks::list_unlinked(&mut tx, auth_user.user_id).await?;

    Ok(UnlinkedBookmarksTemplate { layout, bookmarks })
}

async fn delete_by_id(
    extract::Tx(mut tx): extract::Tx,
    Path(id): Path<Uuid>,
) -> ResponseResult<HeaderMap> {
    db::bookmarks::delete_by_id(&mut tx, id).await?;

    tx.commit().await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Refresh",
        "true".parse().context("Failed to parse header value")?,
    );

    Ok(headers)
}
