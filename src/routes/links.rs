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
    let src_from_db = if let Some(id) = create_link.src {
        Some(db::items::by_id(&mut tx, id).await?)
    } else {
        None
    };
    let dest_from_db = if let Some(id) = create_link.dest {
        Some(db::items::by_id(&mut tx, id).await?)
    } else {
        None
    };
    if let Err(errors) = create_link.validate(&()) {
        return Ok(views::links::CreateLinkTemplate {
            layout,
            errors: errors.into(),
            input: create_link,
            search_results: Vec::new(),
            src_from_db,
            dest_from_db,
        }
        .into_response());
    };

    let mut search_results = Vec::new();
    if let Some(ref search_term) = create_link.search_term {
        search_results = db::items::search(&mut tx, search_term).await?;
    }

    if let (Some(src), Some(dest)) = (&src_from_db, &dest_from_db) {
        if create_link.submitted {
            db::links::insert(
                &mut tx,
                auth_user.user_id,
                CreateLink {
                    src: src.id(),
                    dest: dest.id(),
                },
            )
            .await?;

            tx.commit().await?;

            return Ok(Redirect::to(&src.path()).into_response());
        }
    }

    Ok(CreateLinkTemplate {
        layout,
        errors: Default::default(),
        input: create_link,
        search_results,
        src_from_db,
        dest_from_db,
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
        src_from_db: None,
        dest_from_db: None,
    })
}
