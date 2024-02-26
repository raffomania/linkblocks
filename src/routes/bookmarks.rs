use std::collections::HashMap;

use askama_axum::IntoResponse;
use axum::{
    extract::{Form, Query},
    response::{Redirect, Response},
    routing::get,
    Router,
};
use garde::Validate;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    authentication::AuthUser,
    db, extract,
    forms::bookmarks::CreateBookmark,
    response_error::ResponseResult,
    views::{self, create_bookmark::CreateBookmarkTemplate, layout::LayoutTemplate},
};

pub fn router() -> Router<Pool<Postgres>> {
    Router::new().route("/bookmarks/create", get(get_create).post(post_create))
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
        return Ok(views::create_bookmark::CreateBookmarkTemplate {
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
            ..Default::default()
        },
        selected_parent,
    })
}
