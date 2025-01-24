use activitypub_federation::fetch::object_id::ObjectId;
use anyhow::Result;
use axum::http::StatusCode;

use crate::{
    db,
    forms::users::{Credentials, Login},
    tests::util::test_app::TestApp,
};

#[test_log::test(tokio::test)]
async fn spin_up_two_instances() -> anyhow::Result<()> {
    let mut app_a = TestApp::new().await;
    app_a.create_user("testa", "testpassword").await;

    let mut app_b = TestApp::new().await;
    app_b.create_user("testb", "testpassword").await;

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

    // Check that we can't access instance B with user A
    let login_page = app_b.req().get("/login").await.test_page().await;

    let login_response = login_page
        .expect_status(StatusCode::OK)
        .fill_form("form", &input)
        .await;

    let cookie = login_response.headers().get("Set-Cookie");
    assert!(cookie.is_none());

    Ok(())
}

#[test_log::test(tokio::test)]
async fn can_resolve_user() -> Result<()> {
    let app_a = TestApp::new().await;
    app_a.create_user("testa", "testpassword").await;
    app_a.serve().await;

    let app_b = TestApp::new().await;
    let ap_context = app_b.state.federation_config.to_request_data();

    let user_id = ObjectId::<db::ApUser>::parse("http://localhost:4040/ap/user/testa")?;
    let user = user_id.dereference(&ap_context).await;
    dbg!(&user);
    assert!(user.is_ok());

    let mut tx = app_a.pool.begin().await?;
    let ap_user = db::ap_users::read_local_by_username(&mut tx, "testa").await?;
    let user = ap_user.ap_id.dereference(&ap_context).await;
    assert!(user.is_err());

    Ok(())
}
