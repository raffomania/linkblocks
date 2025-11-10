use activitypub_federation::fetch::webfinger::webfinger_resolve_actor;
use anyhow::Result;
use axum::http::StatusCode;

use crate::{
    db,
    federation::webfinger,
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
    let user = app_a.create_user("testa", "testpassword").await;
    let app_a_ap_user = db::ap_users::read_by_username(
        &mut app_a.pool.begin().await?,
        webfinger::Resource::from_name_and_url(user.username, &app_a.base_url)?,
    )
    .await?;
    app_a.serve().await;

    let app_b = TestApp::new().await;
    let ap_cx_b = app_b.state.federation_config.to_request_data();

    // Check that instance B can resolve user on instance A
    let app_b_ap_user = app_a_ap_user.ap_id.dereference(&ap_cx_b).await?;

    assert_ne!(app_b_ap_user.id, app_a_ap_user.id);
    assert_eq!(app_b_ap_user.username, app_a_ap_user.username);
    assert_eq!(app_b_ap_user.inbox_url, app_a_ap_user.inbox_url);
    assert_eq!(app_b_ap_user.ap_id, app_a_ap_user.ap_id);

    Ok(())
}

#[test_log::test(tokio::test)]
async fn can_resolve_webfinger() -> Result<()> {
    let app = TestApp::new().await;
    let user = app.create_user("testa", "testpassword").await;
    let local_ap_user = db::ap_users::read_by_username(
        &mut app.pool.begin().await?,
        webfinger::Resource::from_name_and_url(user.username, &app.base_url)?,
    )
    .await?;
    app.serve().await;

    let actor: db::ApUser = webfinger_resolve_actor(
        &format!("testa@{}", app.state.federation_config.domain()),
        &app.state.federation_config.to_request_data(),
    )
    .await?;

    assert_eq!(local_ap_user.id, actor.id);

    Ok(())
}
