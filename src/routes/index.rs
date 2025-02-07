use crate::{
  authentication::AuthUser,
  extract,
  response_error::ResponseResult,
  server::AppState,
  views::{self, layout},
};
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(index))
}

async fn index(
  auth_user: AuthUser,
  extract::Tx(mut tx): extract::Tx,
) -> ResponseResult<views::index::Template> {
  Ok(views::index::Template {
    layout: layout::Template::from_db(&mut tx, Some(&auth_user)).await?,
  })
}
