use crate::forms::lists::CreateList;
use crate::server::AppState;
use crate::views::lists::CreateListTemplate;
use crate::{authentication::AuthUser, response_error::ResponseResult};
use crate::{
    db::{self},
    views::{layout::LayoutTemplate, lists::ListTemplate},
};
use crate::{extract, views};
use askama_axum::IntoResponse;
use axum::extract::Path;
use axum::response::Redirect;
use axum::response::Response;
use axum::Form;
use axum::{routing::get, Router};
use garde::Validate;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    let router = Router::new();
    router
        .route("/lists/create", get(get_create).post(post_create))
        .route("/lists/:list_id", get(list))
}

async fn list(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
    Path(list_id): Path<Uuid>,
) -> ResponseResult<ListTemplate> {
    let links = db::links::list_by_list(&mut tx, list_id).await?;
    let list = db::lists::by_id(&mut tx, list_id).await?;

    Ok(ListTemplate {
        layout: LayoutTemplate::from_db(&mut tx, &auth_user).await?,
        links,
        list,
    })
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Form(input): Form<CreateList>,
) -> ResponseResult<Response> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    if let Err(errors) = input.validate() {
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
