use axum::{routing::get, Router};
use sqlx::{Pool, Postgres};

use crate::extract;
use crate::{authentication::AuthUser, response_error::ResponseResult};
use crate::{
    db::{self},
    views::{layout::LayoutTemplate, note::NoteTemplate},
};
use axum::extract::Path;
use uuid::Uuid;

pub fn router() -> Router<Pool<Postgres>> {
    let router = Router::new();
    router.route("/notes/:note_id", get(list))
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
