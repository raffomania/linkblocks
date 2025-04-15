use activitypub_federation::{
    axum::json::FederationJson,
    fetch::webfinger::{Webfinger, build_webfinger_response, extract_webfinger_name},
    protocol::context::WithContext,
    traits::Object,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{
    db::{self},
    extract,
    federation::{self, person::Person},
    response_error::ResponseResult,
    server::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ap/user/{name}", get(get_person))
        .route("/.well-known/webfinger", get(webfinger))
}

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

#[derive(Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

async fn webfinger(
    extract::Tx(mut tx): extract::Tx,
    Query(query): Query<WebfingerQuery>,
    data: federation::Data,
) -> ResponseResult<Json<Webfinger>> {
    let username = extract_webfinger_name(&query.resource, &data)?;
    let ap_id = db::ap_users::read_by_username(&mut tx, username)
        .await?
        .ap_id;
    Ok(Json(build_webfinger_response(
        query.resource,
        ap_id.into_inner(),
    )))
}
