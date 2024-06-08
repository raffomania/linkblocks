use crate::forms::lists::CreateList;
use crate::response_error::ResponseError;
use crate::server::AppState;
use crate::views::lists::CreateListTemplate;
use crate::{authentication::AuthUser, response_error::ResponseResult};
use crate::{
    db::{self},
    views::{layout::LayoutTemplate, lists::ListTemplate},
};
use crate::{extract, views};
use askama_axum::IntoResponse;
use axum::extract::{Path, Query};
use axum::response::Redirect;
use axum::response::Response;
use axum::Form;
use axum::{routing::get, Router};
use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    let router = Router::new();
    router
        .route("/lists/create", get(get_create).post(post_create))
        .route("/lists/toggle_rich_view", get(toggle_rich_view))
        .route("/lists/:list_id", get(list))
}

async fn list(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
    Path(list_id): Path<Uuid>,
) -> ResponseResult<ListTemplate> {
    let links = db::links::list_by_list(&mut tx, list_id).await?;
    let list = db::lists::by_id(&mut tx, list_id).await?;
    let rich_view = list.rich_view;

    Ok(ListTemplate {
        layout: LayoutTemplate::from_db(&mut tx, &auth_user).await?,
        links,
        list,
        rich_view,
    })
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Form(input): Form<CreateList>,
) -> ResponseResult<Response> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    if let Err(errors) = input.validate(&()) {
        return Ok(views::lists::CreateListTemplate {
            layout,
            errors: errors.into(),
            input,
        }
        .into_response());
    };

    let list = db::lists::insert(&mut tx, auth_user.user_id, input).await?;

    tx.commit().await?;

    Ok(Redirect::to(&list.path()).into_response())
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<CreateListTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    Ok(CreateListTemplate {
        layout,
        errors: Default::default(),
        input: CreateList::default(),
    })
}
#[derive(Deserialize)]
struct ToggleRichViewQuery {
    pub list_id: Uuid,
}

async fn toggle_rich_view(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Query(query): Query<ToggleRichViewQuery>,
) -> ResponseResult<Response> {
    let list = db::lists::by_id(&mut tx, query.list_id).await?;
    if list.user_id != auth_user.user_id {
        return Err(ResponseError::NotAuthenticated);
    }
    let new_rich_view = !list.rich_view;

    db::lists::update_rich_view(&mut tx, query.list_id, new_rich_view).await?;

    tx.commit().await?;

    Ok(Redirect::to(&list.path()).into_response())
}
