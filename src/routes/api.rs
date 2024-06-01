use crate::{
    api::{create_api, verify_api},
    authentication::AuthUser,
    db, extract,
    forms::lists::CreateList,
    response_error::ResponseResult,
    server::AppState,
    views::{apikey::ApiKeyTemplate, layout::LayoutTemplate},
};
use anyhow::Context;
use askama_axum::IntoResponse;
use axum::{
    extract::Query,
    response::{Redirect, Response},
    routing::{any, post},
    Json, Router,
};
use serde::Deserialize;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/get_key", any(get_api_key))
        .route("/api/add_bookmark", post(post_add_bookmark))
}
#[derive(Deserialize, Debug, Clone)]
pub struct ApiKeyQuery {
    pub id: Option<String>,
}

async fn get_api_key(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Query(q): Query<ApiKeyQuery>,
) -> ResponseResult<Response> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;
    let api_key = create_api(&mut tx, auth_user.user_id, "write").await?;
    // TODO: Replace with a proper URL
    let url = format!("https://8e048347-accd-40ba-82a9-1f0d661e1d29-00-3dy7faggr9chl.kirk.replit.dev/auth?api_key={}&user_id={}&discord_id={}",api_key.api_key.expect("Failed retrieving API"),api_key.user_id.to_string(),q.id.context("User id not provided")?);
    tx.commit().await?;
    Ok(ApiKeyTemplate { layout, url }.into_response())
}
#[derive(Deserialize, Debug, Clone)]
pub struct AddBookmarkQuery {
    api_key: Option<String>,
    user_id: Option<String>,
    urls: Option<Vec<String>>,
    tag: Option<String>,
}

async fn post_add_bookmark(
    extract::Tx(mut tx): extract::Tx,
    Json(q): Json<AddBookmarkQuery>,
) -> ResponseResult<Response> {
    if !verify_api(
        &mut tx,
        &q.user_id.clone().context("Failed to parse user ID")?,
        &q.api_key.context("Failed to parse API key")?,
    )
    .await?
    {
        return Ok(Redirect::to("/").into_response());
    }
    let user_id = uuid::Uuid::parse_str(&q.user_id.context("Failed to parse user ID")?)
        .context("Failed to parse user ID")?;

    let search_list = db::lists::search(
        &mut tx,
        &q.tag.clone().context("Tag not provided")?,
        user_id.clone(),
    )
    .await?;
    let list = match search_list.len() {
        0 => {
            db::lists::insert(
                &mut tx,
                user_id,
                CreateList {
                    title: q.tag.clone().context("Tag not provided")?,
                    content: Some(format!(
                        "Imported from channel {}",
                        q.tag.context("Tag not provided")?
                    )),
                },
            )
            .await?
        }
        _ => search_list[0].clone(),
    };

    for url in q.urls.clone().context("URLs not provided")? {
        let bookmark = db::bookmarks::insert(
            &mut tx,
            user_id.clone(),
            db::bookmarks::InsertBookmark {
                url: url.clone(),
                title: url,
            },
        )
        .await?;
        db::links::insert(
            &mut tx,
            user_id,
            crate::forms::links::CreateLink {
                src: list.id,
                dest: bookmark.id,
            },
        )
        .await?;
    }

    tx.commit().await?;
    Ok(Redirect::to("/").into_response())
}
