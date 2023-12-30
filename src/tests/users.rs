use axum::http::StatusCode;
use sqlx::{Pool, Postgres};

use crate::{
    schemas::users::{CreateUser, Credentials},
    tests::util::TestApp,
};

#[test_log::test(sqlx::test)]
async fn can_login(pool: Pool<Postgres>) -> anyhow::Result<()> {
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

    let mut app = TestApp::new(pool).await;

    let login_page = app.req().get("/login").await.dom().await;

    let form = login_page.find("form");
    let username = form.find("input[name='username'][type='text'][required]");
    assert!(!username.is_empty());
    let password = form.find("input[name='password'][type='password'][required]");
    assert!(!password.is_empty());

    let creds = Credentials {
        username: "test".to_string(),
        password: "test".to_string(),
    };

    let login_response = app
        .req()
        .expect_status(StatusCode::SEE_OTHER)
        .post("/login", &creds)
        .await;

    let cookie = login_response.headers().get("Set-Cookie").unwrap();
    assert!(!cookie.is_empty());

    Ok(())
}