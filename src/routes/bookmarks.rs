use anyhow::Context;
use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query},
    http::HeaderMap,
    response::{Redirect, Response},
    routing::{delete, get},
    Router,
};

use serde::Deserialize;
use uuid::Uuid;

use crate::{
    authentication::AuthUser,
    db::{self, bookmarks::InsertBookmark},
    extract::{self, qs_form::QsForm},
    forms::{
        bookmarks::CreateBookmark, import::ImportFromOmnivore, links::CreateLink, lists::CreateList,
    },
    import::omnivore::OmnivoreImport,
    response_error::ResponseResult,
    server::AppState,
    views::{
        self,
        bookmarks::{
            CreateBookmarkTemplate, ImportFromOmnivoreTemplate, UnsortedBookmarksTemplate,
        },
        layout::LayoutTemplate,
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bookmarks/create", get(get_create).post(post_create))
        .route("/bookmarks/unsorted", get(get_unsorted))
        .route("/bookmarks/:id", delete(delete_by_id))
        .route(
            "/bookmarks/import_from_omnivore",
            get(get_import_from_omnivore).post(post_import_from_omnivore),
        )
}

async fn post_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    QsForm(input): QsForm<CreateBookmark>,
) -> ResponseResult<Response> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    dbg!(&input);
    let selected_parents = db::lists::list_by_id(&mut tx, &input.parents).await?;

    let search_results = match input.list_search_term.as_ref() {
        None => db::lists::list_recent(&mut tx, auth_user.user_id).await?,
        Some(term) => db::lists::search(&mut tx, term, auth_user.user_id).await?,
    };

    let insert_bookmark = match InsertBookmark::try_from(input.clone()) {
        Err(errors) => {
            return Ok(views::bookmarks::CreateBookmarkTemplate {
                layout,
                errors,
                input,
                selected_parents,
                search_results,
            }
            .into_response());
        }
        Ok(i) => i,
    };

    let bookmark = db::bookmarks::insert(&mut tx, auth_user.user_id, insert_bookmark).await?;

    let mut first_created_parent = Option::None;
    for parent_title in input.create_parents {
        let parent = db::lists::insert(
            &mut tx,
            auth_user.user_id,
            CreateList {
                title: parent_title,
                content: None,
            },
        )
        .await?;
        db::links::insert(
            &mut tx,
            auth_user.user_id,
            CreateLink {
                src: parent.id,
                dest: bookmark.id,
            },
        )
        .await?;

        if first_created_parent.is_none() {
            first_created_parent.replace(parent);
        }
    }

    for parent in input.parents {
        db::links::insert(
            &mut tx,
            auth_user.user_id,
            CreateLink {
                src: parent,
                dest: bookmark.id,
            },
        )
        .await?;
    }

    tx.commit().await?;

    let redirect_dest = match selected_parents.first().or(first_created_parent.as_ref()) {
        Some(parent) => parent.path(),
        None => "/bookmarks/unsorted".to_string(),
    };
    Ok(Redirect::to(&redirect_dest).into_response())
}

#[derive(Deserialize)]
struct CreateBookmarkQuery {
    parent_id: Option<Uuid>,
    url: Option<String>,
    title: Option<String>,
}

async fn get_create(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    Query(query): Query<CreateBookmarkQuery>,
) -> ResponseResult<CreateBookmarkTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    let selected_parent = match query.parent_id {
        Some(id) => Some(db::lists::by_id(&mut tx, id).await?),
        _ => None,
    };

    Ok(CreateBookmarkTemplate {
        layout,
        errors: Default::default(),
        input: CreateBookmark {
            parents: Vec::new(),
            url: query.url.unwrap_or_default(),
            title: query.title.unwrap_or_default(),
            ..Default::default()
        },
        selected_parents: Vec::from_iter(selected_parent.into_iter()),
        search_results: db::lists::list_recent(&mut tx, auth_user.user_id).await?,
    })
}

async fn get_unsorted(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<UnsortedBookmarksTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;
    let bookmarks = db::bookmarks::list_unsorted(&mut tx, auth_user.user_id).await?;

    Ok(UnsortedBookmarksTemplate { layout, bookmarks })
}

async fn delete_by_id(
    extract::Tx(mut tx): extract::Tx,
    Path(id): Path<Uuid>,
) -> ResponseResult<HeaderMap> {
    db::bookmarks::delete_by_id(&mut tx, id).await?;

    tx.commit().await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Refresh",
        "true".parse().context("Failed to parse header value")?,
    );

    Ok(headers)
}

async fn post_import_from_omnivore(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
    QsForm(import): QsForm<ImportFromOmnivore>,
) -> ResponseResult<Response> {
    let api_token = import.api_token;
    let omnivore_graphql_endpoint_url = "https://api-prod.omnivore.app/api/graphql".to_string();

    let client = OmnivoreImport::new(api_token, omnivore_graphql_endpoint_url);
    let result = client
        .get_articles(
            Some(1000),
            None,
            "markdown".to_string(),
            "in:inbox".to_string(),
            false,
        )
        .await
        .expect("Failed to get articles");
    let parent = db::lists::insert(
        &mut tx,
        auth_user.user_id,
        CreateList {
            title: "Omnivore".to_string(),
            content: Some("Imported from Omnivore".to_string()),
        },
    )
    .await?;
    for article in result["data"]["search"]["edges"].as_array().unwrap() {
        let title = article["node"]["title"].as_str().unwrap();
        let url = article["node"]["originalArticleUrl"].as_str().unwrap();
        let bookmark = db::bookmarks::insert(
            &mut tx,
            auth_user.user_id,
            InsertBookmark {
                url: url.to_string(),
                title: title.to_string(),
            },
        )
        .await?;
        db::links::insert(
            &mut tx,
            auth_user.user_id,
            CreateLink {
                src: parent.id,
                dest: bookmark.id,
            },
        )
        .await?;
    }
    tx.commit().await?;

    Ok("Imported from Omnivore".into_response())
}

async fn get_import_from_omnivore(
    extract::Tx(mut tx): extract::Tx,
    auth_user: AuthUser,
) -> ResponseResult<ImportFromOmnivoreTemplate> {
    let layout = LayoutTemplate::from_db(&mut tx, &auth_user).await?;

    Ok(ImportFromOmnivoreTemplate { layout })
}
