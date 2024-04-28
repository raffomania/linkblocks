use crate::authentication::Authenticated;
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
        .route("/lists/:list_id", get(list))
        .route("/lists/toggle_publicity", get(toggle_publicity))
}


async fn list(
    authenticated: Authenticated,
    extract::Tx(mut tx): extract::Tx,
    Path(list_id): Path<Uuid>,
) -> ResponseResult<ListTemplate> {
// ) -> ResponseResult<TypeList<ListTemplate, PublicListTemplate>> {
    let links = db::links::list_by_list(&mut tx, list_id).await?;
    let list = db::lists::by_id(&mut tx, list_id.clone()).await?;
    
    let auth_user = authenticated.auth_user;

    match auth_user {
        Some(ref user) => {
            if list.prvate && list.user_id != user.user_id{
                return Err(ResponseError::NotFound)
            }
        },
        None => {
            if list.prvate{
                return Err(ResponseError::NotFound)
            }
        }
    }

    return Ok(ListTemplate {
        layout: LayoutTemplate::from_db(&mut tx, auth_user).await?,
        links,
        list,
    })

}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Form(input): Form<CreateList>,
) -> ResponseResult<Response> {
    let user_id = auth_user.user_id;
    let layout = LayoutTemplate::from_db(&mut tx, Some(auth_user)).await?;

    if let Err(errors) = input.validate(&()) {
        return Ok(views::lists::CreateListTemplate {
            layout,
            errors: errors.into(),
            input,
        }
        .into_response());
    };

    let list = db::lists::insert(&mut tx, user_id, input).await?;

    tx.commit().await?;

    Ok(Redirect::to(&list.path()).into_response())
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<CreateListTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, Some(auth_user)).await?;

    Ok(CreateListTemplate {
        layout,
        errors: Default::default(),
        input: CreateList::default(),
    })
}

#[derive(Deserialize)]
struct TogglePublicityQuery {
    list_id: Uuid,
}
async fn toggle_publicity(
    authenticated: Authenticated,
    extract::Tx(mut tx): extract::Tx,
    Query(query): Query<TogglePublicityQuery>,
) -> ResponseResult<Response> {
    let list = db::lists::by_id(&mut tx, query.list_id.clone()).await?;
    let auth_user = authenticated.auth_user;

    match auth_user {
        Some(ref user) => {
            if list.user_id != user.user_id {
                return Err(ResponseError::NotFound)
            }
        },
        None => {
            return Err(ResponseError::NotFound)
        }
    }
    
    if list.prvate {
        db::lists::set_private(&mut tx, query.list_id,false).await?;
    } else {
        db::lists::set_private(&mut tx, query.list_id,true).await?;
    }

    tx.commit().await?;

    Ok(Redirect::to(&list.path()).into_response())
}