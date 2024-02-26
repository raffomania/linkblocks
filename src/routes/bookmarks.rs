use askama_axum::IntoResponse;
use axum::{
    extract::{Form, Query},
    response::{Redirect, Response},
    routing::get,
    Router,
};
use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    authentication::AuthUser,
    db, extract,
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
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Form(input): Form<CreateBookmark>,
) -> ResponseResult<Response> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    let selected_parent = match input.parent {
        Some(id) => Some(db::items::by_id(&mut tx, id).await?),
        None => None,
    };

    if let Err(errors) = input.validate(&()) {
        return Ok(views::bookmarks::CreateBookmarkTemplate {
            layout,
            errors: errors.into(),
            input,
            selected_parent,
        }
        .into_response());
    };

    let bookmark = db::bookmarks::insert(&mut tx, auth_user.user_id, input).await?;

    tx.commit().await?;

    let redirect_dest = match selected_parent {
        Some(parent) => parent.path(),
        None => bookmark.path(),
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
        Some(id) => Some(db::items::by_id(&mut tx, id).await?),
        _ => None,
    };

    Ok(CreateBookmarkTemplate {
        layout,
        errors: Default::default(),
        input: CreateBookmark {
            parent: selected_parent.as_ref().map(|p| p.id()),
            url: query.url.unwrap_or_default(),
            title: query.title.unwrap_or_default(),
        },
        selected_parent,
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
