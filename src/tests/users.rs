use axum::http::{StatusCode, header};

use crate::{
    db::ap_users,
    forms::users::{CreateOidcUser, Credentials, Login},
    tests::util::test_app::TestApp,
};

#[test_log::test(tokio::test)]
async fn can_login() -> anyhow::Result<()> {
    let mut app = TestApp::new().await;
    app.create_user("test", "testpassword").await;

    let login_page = app.req().get("/login").await.test_page().await;
    insta::assert_snapshot!(login_page.dom.htmls());

    let input = Login {
        credentials: Credentials {
            username: "test".to_string(),
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
    let cookie = cookie.to_str()?.split_once(';').unwrap().0;

    app.req().header(header::COOKIE, cookie).get("/").await;

    Ok(())
}

#[test_log::test(tokio::test)]
async fn insert_oidc_creates_ap_user() -> anyhow::Result<()> {
    let app = TestApp::new().await;
    let mut tx = app.pool.begin().await?;

    let create_oidc_user = CreateOidcUser {
        oidc_id: "test_oidc_id".to_string(),
        email: "test@example.com".to_string(),
        username: "test_oidc_user".to_string(),
    };

    // Insert OIDC user
    let user = crate::db::users::insert_oidc(&mut tx, create_oidc_user, &app.base_url).await?;

    // Verify that the user has a valid ap_user_id
    let ap_user = ap_users::read_by_id(&mut tx, user.ap_user_id).await?;

    // Verify AP user properties
    assert_eq!(ap_user.username, "test_oidc_user");

    Ok(())
}
