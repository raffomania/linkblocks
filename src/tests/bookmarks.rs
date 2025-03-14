use crate::tests::util::test_app::TestApp;

#[test_log::test(tokio::test)]
async fn get_unsorted_bookmarks() -> anyhow::Result<()> {
    let mut app = TestApp::new().await;
    app.create_test_user().await;
    app.login_test_user().await;

    let unsorted_bookmarks = app.req().get("/bookmarks/unsorted").await.test_page().await;

    insta::assert_snapshot!(unsorted_bookmarks.dom.htmls());

    Ok(())
}
