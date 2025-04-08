use crate::form_errors::FormErrors;
use crate::forms::lists::{CreateList, EditListPinned, EditListPrivate};
use crate::htmf_response::HtmfResponse;
use crate::response_error::ResponseError;
use crate::server::AppState;
use crate::views::lists::{EditListTitleTemplate, UnpinnedListsTemplate};
use crate::{authentication::AuthUser, response_error::ResponseResult};
use crate::{
    db::{self},
    views::layout,
};
use crate::{extract, forms, views};
use axum::Form;
use axum::extract::Path;
use axum::response::Response;
use axum::response::{IntoResponse, Redirect};
use axum::routing::post;
use axum::{Router, routing::get};
use garde::Validate;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    let router: Router<AppState> = Router::new();
    router
        .route("/lists/create", get(get_create).post(post_create))
        .route("/lists/{list_id}", get(list))
        .route("/lists/{list_id}/edit_private", post(edit_private))
        .route("/lists/{list_id}/edit_title", post(post_edit_title))
        .route("/lists/{list_id}/edit_title", get(get_edit_title))
        .route("/lists/{list_id}/edit_pinned", post(edit_pinned))
        .route("/lists/unpinned", get(list_unpinned))
}

async fn list(
    auth_user: Option<AuthUser>,
    extract::Tx(mut tx): extract::Tx,
    Path(list_id): Path<Uuid>,
) -> ResponseResult<HtmfResponse> {
    let links =
        db::links::list_by_list(&mut tx, list_id, auth_user.as_ref().map(|u| u.user_id)).await?;
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

    Ok(HtmfResponse(views::list::view(&views::list::Data {
        layout: layout::Template::from_db(&mut tx, auth_user.as_ref()).await?,
        links,
        list,
        metadata: db::lists::metadata_by_id(&mut tx, list_id).await?,
    })))
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Form(input): Form<CreateList>,
) -> ResponseResult<Response> {
    let user_id = auth_user.user_id;
    let layout = layout::Template::from_db(&mut tx, Some(&auth_user)).await?;

    if let Err(errors) = input.validate() {
        return Ok(views::create_list::view(&views::create_list::Data {
            layout,
            input,
            errors: errors.into(),
        })
        .to_html()
        .into_response());
    };

    let list = db::lists::insert(&mut tx, user_id, input).await?;

    tx.commit().await?;

    Ok(Redirect::to(&list.path()).into_response())
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<HtmfResponse> {
    let layout = layout::Template::from_db(&mut tx, Some(&auth_user)).await?;

    Ok(HtmfResponse(views::create_list::view(
        &views::create_list::Data {
            layout,
            input: CreateList::default(),
            errors: FormErrors::default(),
        },
    )))
}

async fn get_edit_title(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Path(list_id): Path<Uuid>,
) -> ResponseResult<EditListTitleTemplate> {
    let list = db::lists::by_id(&mut tx, list_id).await?;

    if list.user_id != auth_user.user_id {
        return Err(ResponseError::NotFound);
    }

    let layout = layout::Template::from_db(&mut tx, Some(&auth_user)).await?;

    Ok(EditListTitleTemplate {
        layout,
        errors: FormErrors::default(),
        input: forms::lists::EditTitle::default(),
        list_id,
    })
}

async fn post_edit_title(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
    Path(list_id): Path<Uuid>,
    Form(input): Form<forms::lists::EditTitle>,
) -> ResponseResult<Response> {
    let list = db::lists::by_id(&mut tx, list_id).await?;

    if list.user_id != auth_user.user_id {
        return Err(ResponseError::NotFound);
    }

    db::lists::edit_title(&mut tx, list_id, input.title).await?;

    tx.commit().await?;

    Ok(Redirect::to(&list.path()).into_response())
}

async fn edit_private(
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

async fn edit_pinned(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
    Path(list_id): Path<Uuid>,
    Form(input): Form<EditListPinned>,
) -> ResponseResult<Response> {
    let list = db::lists::by_id(&mut tx, list_id).await?;

    if list.user_id != auth_user.user_id {
        return Err(ResponseError::NotFound);
    }

    db::lists::set_pinned(&mut tx, list_id, input.pinned).await?;

    tx.commit().await?;

    Ok(Redirect::to(&list.path()).into_response())
}

// TODO colocate this with view and db code
async fn list_unpinned(
    auth_user: AuthUser,
    extract::Tx(mut tx): extract::Tx,
) -> ResponseResult<UnpinnedListsTemplate> {
    let lists = db::lists::list_unpinned(&mut tx, auth_user.user_id).await?;

    Ok(UnpinnedListsTemplate {
        layout: layout::Template::from_db(&mut tx, Some(&auth_user)).await?,
        lists,
    })
}
