use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    Router,
};
use http_body_util::BodyExt;
use sqlx::{Pool, Postgres};
use tower::Service;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use visdom::Vis;

use crate::schemas::users::{CreateUser, Credentials}; // for `call`, `oneshot`, and `ready`

#[sqlx::test]
async fn can_login(pool: Pool<Postgres>) -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut tx = pool.begin().await?;
    crate::db::users::create_user_if_not_exists(
        &mut tx,
        CreateUser {
            username: "test".to_string(),
            password: "test".to_string(),
        },
    )
    .await?;
    tx.commit().await?;

    let mut app = crate::server::app(pool).await.unwrap();

    let response = <Router as tower::ServiceExt<Request<Body>>>::ready(&mut app)
        .await?
        .call(Request::builder().uri("/login").body(Body::empty())?)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = String::from_utf8(response.into_body().collect().await?.to_bytes().to_vec())?;
    let dom = Vis::load(body).expect("Failed to parse HTML");
    let form = dom.find("form");
    let username = form.find("input[name='username'][type='text'][required]");
    assert!(!username.is_empty());
    let password = form.find("input[name='password'][type='password'][required]");
    assert!(!password.is_empty());

    let creds = axum::Form(Credentials {
        username: "test".to_string(),
        password: "test".to_string(),
    })
    .into_response()
    .into_body();

    let response = <Router as tower::ServiceExt<Request<Body>>>::ready(&mut app)
        .await?
        .call(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(creds)?,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let cookie = response.headers().get("Set-Cookie").unwrap();
    dbg!(cookie);

    Ok(())
}
