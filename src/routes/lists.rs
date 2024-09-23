use crate::form_errors::FormErrors;
use crate::forms::lists::{CreateList, EditListPrivate};
use crate::response_error::ResponseError;
use crate::server::AppState;
use crate::views::lists::CreateListTemplate;
use crate::{authentication::AuthUser, response_error::ResponseResult};
use crate::{
    db::{self},
    views::{layout, lists::ListTemplate},
};
use crate::{extract, views};
use askama_axum::IntoResponse;
use axum::extract::Path;
use axum::response::Redirect;
use axum::response::Response;
use axum::routing::post;
use axum::Form;
use axum::{routing::get, Router};
use garde::Validate;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    let router = Router::new();
    router
        .route("/lists/create", get(get_create).post(post_create))
        .route("/lists/:list_id", get(list))
        .route("/lists/:list_id/edit_private", post(toggle_publicity))
}

async fn list(
    auth_user: Option<AuthUser>,
    extract::Tx(mut tx): extract::Tx,
    Path(list_id): Path<Uuid>,
) -> ResponseResult<ListTemplate> {
    let links = db::links::list_by_list(&mut tx, list_id).await?;
    let list = db::lists::by_id(&mut tx, list_id).await?;

    match auth_user {
        Some(ref user) => {
            if list.private && list.user_id != user.user_id {
                return Err(ResponseError::NotFound);
            }
        }
        None => {
            if list.private {
                return Err(ResponseError::NotFound);
            }
        }
    }

    Ok(ListTemplate {
        layout: layout::Template::from_db(&mut tx, auth_user.as_ref()).await?,
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
    let layout = layout::Template::from_db(&mut tx, Some(&auth_user)).await?;

    if let Err(errors) = input.validate() {
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
    let layout = layout::Template::from_db(&mut tx, Some(&auth_user)).await?;

    Ok(CreateListTemplate {
        layout,
        errors: FormErrors::default(),
        input: CreateList::default(),
    })
}

async fn toggle_publicity(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
    Path(list_id): Path<Uuid>,
    Form(input): Form<EditListPrivate>,
) -> ResponseResult<Response> {
    let list = db::lists::by_id(&mut tx, list_id).await?;

    if list.user_id != auth_user.user_id {
        return Err(ResponseError::NotFound);
    }

    db::lists::set_private(&mut tx, list_id, input.private).await?;

    tx.commit().await?;

    Ok(Redirect::to(&list.path()).into_response())
}
