use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::{Pool, Postgres};
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

#[sqlx::test]
async fn index(pool: Pool<Postgres>) {
    let app = crate::server::app(pool).await.unwrap();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}
