use activitypub_federation::{
    axum::json::FederationJson, protocol::context::WithContext, traits::Object,
};
use axum::{
    Router, debug_handler,
    extract::{Path, State},
    routing::get,
};

use crate::{
    db::{self},
    extract,
    federation::person::Person,
    response_error::ResponseResult,
    server::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/ap/user/{name}", get(get_person))
}

#[debug_handler]
async fn get_person(
    extract::Tx(mut tx): extract::Tx,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ResponseResult<FederationJson<WithContext<Person>>> {
    let ap_user = db::ap_users::read_by_username(&mut tx, &name).await?;
    let json_person = ap_user
        .into_json(&state.federation_config.to_request_data())
        .await?;
    Ok(FederationJson(WithContext::new_default(json_person)))
}
