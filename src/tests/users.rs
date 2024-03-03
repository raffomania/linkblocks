use axum::http::{header, StatusCode};
use sqlx::{Pool, Postgres};

use crate::{
    forms::users::{CreateUser, Credentials, Login},
    tests::util::{dom::assert_form_matches, TestApp},
};

#[test_log::test(sqlx::test)]
async fn can_login(pool: Pool<Postgres>) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    crate::db::users::create_user_if_not_exists(
        &mut tx,
        CreateUser {
            username: "test".to_string(),
            password: "testpassword".to_string(),
        },
    )
    .await?;
    tx.commit().await?;

    let mut app = TestApp::new(pool).await;

    let login_page = app.req().get("/login").await.dom().await;

    let form = login_page.find("form");

    let input = Login {
        credentials: Credentials {
            username: "test".to_string(),
            password: "testpassword".to_string(),
        },
        previous_uri: None,
    };
    assert_form_matches(form, &input);

    dbg!(serde_qs::to_string(&input)?);

    let login_response = app
        .req()
        .expect_status(StatusCode::SEE_OTHER)
        .post("/login", &input)
        .await;

    let cookie = login_response.headers().get("Set-Cookie").unwrap();
    assert!(!cookie.is_empty());
    let cookie = cookie.to_str()?.split_once(';').unwrap().0;

    // Check that we can access the index using the auth cookie
    app.req().header(header::COOKIE, cookie).get("/").await;

    Ok(())
}
