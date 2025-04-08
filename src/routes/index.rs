use axum::{Router, routing::get};

use crate::{
    authentication::AuthUser,
    extract,
    htmf_response::HtmfResponse,
    response_error::ResponseResult,
    server::AppState,
    views::{self, layout},
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(index))
}

async fn index(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
) -> ResponseResult<HtmfResponse> {
    Ok(views::index::view(&layout::Template::from_db(&mut tx, Some(&auth_user)).await?).into())
}
