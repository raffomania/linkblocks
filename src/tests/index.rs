use axum::http::StatusCode;
use sqlx::{Pool, Postgres};

use crate::tests::util::test_app::TestApp;

#[test_log::test(sqlx::test)]
async fn index(pool: Pool<Postgres>) -> anyhow::Result<()> {
    let mut app = TestApp::new(pool).await;

    app.req()
        .expect_status(StatusCode::SEE_OTHER)
        .get("/")
        .await;

    Ok(())
}
