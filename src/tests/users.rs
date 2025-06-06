use axum::http::{StatusCode, header};

use crate::{
    forms::users::{Credentials, Login},
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
