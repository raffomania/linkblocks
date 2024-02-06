use crate::{
    app_error::AppResult,
    authentication::AuthUser,
    db::{self, ExtractTx},
    views::{layout::LayoutTemplate, list::ListTemplate},
};
use axum::{extract::Path, routing::get, Router};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub fn router() -> Router<Pool<Postgres>> {
    Router::new().route("/lists/:list_id", get(list))
}

async fn list(
    auth_user: AuthUser,
    ExtractTx(mut tx): ExtractTx,
    Path(list_id): Path<Uuid>,
) -> AppResult<ListTemplate> {
    let user = db::users::by_id(&mut tx, auth_user.user_id).await?;
    let links = db::links::list_by_list(&mut tx, list_id).await?;
    let lists = db::lists::list_by_user(&mut tx, auth_user.user_id).await?;
    let list = db::lists::by_id(&mut tx, list_id).await?;

    Ok(ListTemplate {
        layout: LayoutTemplate {
            logged_in_username: user.username,
            lists,
        },
        links,
        list,
    })
}
