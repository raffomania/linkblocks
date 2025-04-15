use activitypub_federation::fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor};
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
    let user = app_a.create_user("testa", "testpassword").await;
    let app_a_ap_user =
        db::ap_users::read_by_username(&mut app_a.pool.begin().await?, &user.username).await?;
    app_a.serve().await;

    let app_b = TestApp::new().await;
    let ap_cx_b = app_b.state.federation_config.to_request_data();

    // Check that instance B can resolve user on instance A
    let user_id =
        ObjectId::<db::ApUser>::parse(&format!("{}ap/user/{}", app_a.base_url, user.username))?;
    assert_eq!(user_id, app_a_ap_user.ap_id);
    let resolved_ap_user_1 = user_id.dereference(&ap_cx_b).await?;
    let resolved_ap_user_2 = app_a_ap_user.ap_id.dereference(&ap_cx_b).await?;
    assert_eq!(resolved_ap_user_1.ap_id, resolved_ap_user_2.ap_id);
    assert_eq!(resolved_ap_user_1.id, resolved_ap_user_2.id);

    Ok(())
}

#[test_log::test(tokio::test)]
async fn can_resolve_webfinger() -> Result<()> {
    let app = TestApp::new().await;
    let user = app.create_user("testa", "testpassword").await;
    let local_ap_user =
        db::ap_users::read_by_username(&mut app.pool.begin().await?, &user.username).await?;
    app.serve().await;

    let actor: db::ApUser = webfinger_resolve_actor(
        &format!("testa@{}", app.state.federation_config.domain()),
        &app.state.federation_config.to_request_data(),
    )
    .await?;

    assert_eq!(local_ap_user.id, actor.id);

    Ok(())
}
