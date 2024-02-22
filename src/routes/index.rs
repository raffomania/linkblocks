use crate::{
    authentication::AuthUser,
    extract,
    response_error::ResponseResult,
    views::{index::IndexTemplate, layout::LayoutTemplate},
};
use axum::{routing::get, Router};
use sqlx::{Pool, Postgres};

pub fn router() -> Router<Pool<Postgres>> {
    Router::new().route("/", get(index))
}

async fn index(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
) -> ResponseResult<IndexTemplate> {
    Ok(IndexTemplate {
        layout: LayoutTemplate::from_db(&mut tx, &auth_user).await?,
    })
}
