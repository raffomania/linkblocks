use axum::http::{header, StatusCode};

use crate::{
    forms::users::{CreateUser, Credentials, Login},
    tests::util::test_app::TestApp,
};

#[test_log::test(tokio::test)]
async fn spin_up_two_instances() -> anyhow::Result<()> {
    let pool_a = super::util::db::new_pool().await;
    let mut tx = pool_a.begin().await?;
    crate::db::users::create_user_if_not_exists(
        &mut tx,
        CreateUser {
            username: "testa".to_string(),
            password: "testpassword".to_string(),
        },
    )
    .await?;
    tx.commit().await?;

    let mut app_a = TestApp::new(pool_a.clone()).await;

    let pool_b = super::util::db::new_pool().await;
    let mut tx = pool_b.begin().await?;
    crate::db::users::create_user_if_not_exists(
        &mut tx,
        CreateUser {
            username: "testb".to_string(),
            password: "testpassword".to_string(),
        },
    )
    .await?;
    tx.commit().await?;
    let mut app_b = TestApp::new(pool_b).await;

    let login_page = app_a.req().get("/login").await.test_page().await;

    let input = Login {
        credentials: Credentials {
            username: "testa".to_string(),
            password: "testpassword".to_string(),
        },
        previous_uri: None,
    };

    let login_response = login_page
        .expect_status(StatusCode::SEE_OTHER)
        .fill_form("form", &input)
        .await;

    let cookie = login_response.headers().get("Set-Cookie").unwrap();
    assert!(!cookie.is_empty());
    let cookie = cookie.to_str()?.split_once(';').unwrap().0.to_string();

    // Check that we can access the index using the auth cookie
    app_a.req().header(header::COOKIE, cookie).get("/").await;

    // Check that we can't access instance B with user A
    let login_page = app_b.req().get("/login").await.test_page().await;

    let input = Login {
        credentials: Credentials {
            username: "testa".to_string(),
            password: "testpassword".to_string(),
        },
        previous_uri: None,
    };

    let login_response = login_page
        .expect_status(StatusCode::OK)
        .fill_form("form", &input)
        .await;

    let cookie = login_response.headers().get("Set-Cookie");
    assert!(cookie.is_none());

    Ok(())
}
