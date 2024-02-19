use askama_axum::IntoResponse;
use axum::{
    extract::Form,
    response::{Redirect, Response},
    routing::get,
    Router,
};
use garde::Validate;
use sqlx::{Pool, Postgres};

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
    Form(create_bookmark): Form<CreateBookmark>,
) -> ResponseResult<Response> {
    let pinned_notes = db::notes::list_pinned_by_user(&mut tx, auth_user.user_id).await?;
    let user = db::users::by_id(&mut tx, auth_user.user_id).await?;
    let layout = LayoutTemplate {
        logged_in_username: user.username,
        notes: pinned_notes,
    };
    if let Err(errors) = create_bookmark.validate(&()) {
        return Ok(views::create_bookmark::CreateBookmarkTemplate {
            layout,
            errors: errors.into(),
            input: create_bookmark,
        }
        .into_response());
    };

    db::bookmarks::insert(&mut tx, auth_user.user_id, create_bookmark).await?;

    Ok(Redirect::to("/").into_response())
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<CreateBookmarkTemplate> {
    let pinned_notes = db::notes::list_pinned_by_user(&mut tx, auth_user.user_id).await?;
    let user = db::users::by_id(&mut tx, auth_user.user_id).await?;
    let layout = LayoutTemplate {
        logged_in_username: user.username,
        notes: pinned_notes,
    };

    Ok(CreateBookmarkTemplate {
        layout,
        errors: Default::default(),
        input: Default::default(),
    })
}
