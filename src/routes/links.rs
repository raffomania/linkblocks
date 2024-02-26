use askama_axum::IntoResponse;
use axum::{
    extract::Query,
    response::{Redirect, Response},
    routing::get,
    Router,
};
use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    authentication::AuthUser,
    db,
    extract::{self, qs_form::QsForm},
    forms::links::{CreateLink, PartialCreateLink},
    response_error::ResponseResult,
    server::AppState,
    views::{self, layout::LayoutTemplate, links::CreateLinkTemplate},
};

pub fn router() -> Router<AppState> {
    Router::new().route("/links/create", get(get_create).post(post_create))
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    // TODO handle failed extractors in forms better
    QsForm(input): QsForm<PartialCreateLink>,
) -> ResponseResult<Response> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;
    let src_from_db = match input.src {
        Some(id) => Some(db::items::by_id(&mut tx, id).await?),
        None => None,
    };
    let dest_from_db = match input.dest {
        Some(id) => Some(db::items::by_id(&mut tx, id).await?),
        None => None,
    };

    if let Err(errors) = input.validate(&()) {
        return Ok(views::links::CreateLinkTemplate {
            layout,
            errors: errors.into(),
            input,
            search_results: Vec::new(),
            src_from_db,
            dest_from_db,
        }
        .into_response());
    };

    let search_term = match (input.src, input.dest) {
        (None, _) => input.search_term_src.as_ref(),
        (Some(_), None) => input.search_term_dest.as_ref(),
        _ => None,
    };

    let mut search_results = Vec::new();
    if let Some(search_term) = search_term {
        search_results = db::items::search(&mut tx, search_term).await?;
    }

    if let (Some(src), Some(dest), true) = (&src_from_db, &dest_from_db, input.submitted) {
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

    Ok(CreateLinkTemplate {
        layout,
        errors: Default::default(),
        input,
        search_results,
        src_from_db,
        dest_from_db,
    }
    .into_response())
}

#[derive(Deserialize)]
struct CreateLinkQueryString {
    src_id: Option<Uuid>,
    dest_id: Option<Uuid>,
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Query(query): Query<CreateLinkQueryString>,
) -> ResponseResult<CreateLinkTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    let src = match query.src_id {
        Some(id) => Some(db::items::by_id(&mut tx, id).await?),
        _ => None,
    };

    let dest = match query.dest_id {
        Some(id) => Some(db::items::by_id(&mut tx, id).await?),
        _ => None,
    };

    Ok(CreateLinkTemplate {
        layout,
        errors: Default::default(),
        input: PartialCreateLink {
            src: src.as_ref().map(|item| item.id()),
            dest: dest.as_ref().map(|item| item.id()),
            ..PartialCreateLink::default()
        },
        search_results: Vec::new(),
        src_from_db: src,
        dest_from_db: dest,
    })
}
