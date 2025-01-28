use axum::http::StatusCode;

use crate::tests::util::test_app::TestApp;

#[test_log::test(tokio::test)]
async fn index() -> anyhow::Result<()> {
    let mut app = TestApp::new().await;

    app.req()
        .expect_status(StatusCode::SEE_OTHER)
        .get("/")
        .await;

    Ok(())
}
