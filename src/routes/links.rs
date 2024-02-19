use askama_axum::IntoResponse;
use axum::{
    response::{Redirect, Response},
    routing::get,
    Router,
};
use garde::Validate;
use sqlx::{Pool, Postgres};

use crate::{
    authentication::AuthUser,
    db,
    extract::{self, qs_form::QsForm},
    forms::links::{CreateLink, PartialCreateLink},
    response_error::ResponseResult,
    views::{self, layout::LayoutTemplate, links::CreateLinkTemplate},
};

pub fn router() -> Router<Pool<Postgres>> {
    Router::new().route("/links/create", get(get_create).post(post_create))
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    // TODO handle failed extractors in forms better
    QsForm(create_link): QsForm<PartialCreateLink>,
) -> ResponseResult<Response> {
    let pinned_notes = db::notes::list_pinned_by_user(&mut tx, auth_user.user_id).await?;
    let user = db::users::by_id(&mut tx, auth_user.user_id).await?;
    let layout = LayoutTemplate {
        logged_in_username: user.username,
        notes: pinned_notes,
    };
    if let Err(errors) = create_link.validate(&()) {
        return Ok(views::links::CreateLinkTemplate {
            layout,
            errors: errors.into(),
            input: create_link,
            search_results: Vec::new(),
            selected_src: todo!(),
            selected_dest: todo!(),
        }
        .into_response());
    };

    let mut search_results = Vec::new();
    if let Some(ref search_term) = create_link.search_term {
        search_results = db::links::search_linkable_items(&mut tx, search_term).await?;
    }

    if let (Some(src), Some(dest)) = (create_link.src.clone(), create_link.dest.clone()) {
        if create_link.submitted {
            db::links::insert(&mut tx, auth_user.user_id, CreateLink { src, dest }).await?;
            return Ok(Redirect::to("/").into_response());
        }
    }

    Ok(CreateLinkTemplate {
        layout,
        errors: Default::default(),
        input: create_link,
        search_results,
        selected_src: None,
        selected_dest: None,
    }
    .into_response())
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<CreateLinkTemplate> {
    let pinned_notes = db::notes::list_pinned_by_user(&mut tx, auth_user.user_id).await?;
    let user = db::users::by_id(&mut tx, auth_user.user_id).await?;
    let layout = LayoutTemplate {
        logged_in_username: user.username,
        notes: pinned_notes,
    };

    Ok(CreateLinkTemplate {
        layout,
        errors: Default::default(),
        input: PartialCreateLink::default(),
        search_results: Vec::new(),
        selected_src: None,
        selected_dest: None,
    })
}
