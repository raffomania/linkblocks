use askama_axum::IntoResponse;
use axum::response::Redirect;
use axum::response::Response;
use axum::Form;
use axum::{routing::get, Router};
use garde::Validate;
use sqlx::{Pool, Postgres};

use crate::forms::notes::CreateNote;
use crate::views::notes::CreateNoteTemplate;
use crate::{authentication::AuthUser, response_error::ResponseResult};
use crate::{
    db::{self},
    views::{layout::LayoutTemplate, notes::NoteTemplate},
};
use crate::{extract, views};
use axum::extract::Path;
use uuid::Uuid;

pub fn router() -> Router<Pool<Postgres>> {
    let router = Router::new();
    router
        .route("/notes/create", get(get_create).post(post_create))
        .route("/notes/:note_id", get(list))
}

async fn list(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
    Path(note_id): Path<Uuid>,
) -> ResponseResult<NoteTemplate> {
    let links = db::links::list_by_note(&mut tx, note_id).await?;
    let note = db::notes::by_id(&mut tx, note_id).await?;

    Ok(NoteTemplate {
        layout: LayoutTemplate::from_db(&mut tx, &auth_user).await?,
        links,
        note,
    })
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Form(input): Form<CreateNote>,
) -> ResponseResult<Response> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    if let Err(errors) = input.validate(&()) {
        return Ok(views::notes::CreateNoteTemplate {
            layout,
            errors: errors.into(),
            input,
        }
        .into_response());
    };

    let note = db::notes::insert(&mut tx, auth_user.user_id, input).await?;

    tx.commit().await?;

    Ok(Redirect::to(&note.path()).into_response())
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<CreateNoteTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    Ok(CreateNoteTemplate {
        layout,
        errors: Default::default(),
        input: CreateNote::default(),
    })
}
