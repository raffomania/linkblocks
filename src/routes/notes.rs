use axum::{routing::get, Router};
use sqlx::{Pool, Postgres};

use crate::{authentication::AuthUser, db::ExtractTx, response_error::ResponseResult};
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
    ExtractTx(mut tx): ExtractTx,
    Path(note_id): Path<Uuid>,
) -> ResponseResult<NoteTemplate> {
    let user = db::users::by_id(&mut tx, auth_user.user_id).await?;
    let links = db::links::list_by_note(&mut tx, note_id).await?;
    let pinned_notes = db::notes::list_pinned_by_user(&mut tx, auth_user.user_id).await?;
    let note = db::notes::by_id(&mut tx, note_id).await?;

    Ok(NoteTemplate {
        layout: LayoutTemplate {
            logged_in_username: user.username,
            notes: pinned_notes,
        },
        links,
        note,
    })
}
