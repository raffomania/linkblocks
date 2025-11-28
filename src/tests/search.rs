use pretty_assertions::assert_eq;

use crate::{
    db::{self, bookmarks::InsertBookmark},
    routes::search::SearchQuery,
    tests::util::test_app::TestApp,
};

#[test_log::test(tokio::test)]
async fn search_finds_bookmarks_with_various_queries() -> anyhow::Result<()> {
    let mut app = TestApp::new().await;
    let user = app.create_test_user().await;
    app.login_test_user().await;

    // Create bookmarks with different titles (6 Rust bookmarks to trigger
    // pagination since page size is 4, plus other bookmarks)
    let mut tx = app.tx().await;
    let titles = vec![
        "Learning Rust Programming",
        "Advanced Rust Patterns",
        "Rust Async Programming",
        "Rust Performance Optimization",
        "Rust Web Development",
        "Rust Macros Guide",
        // Non-Rust bookmarks to ensure search filtering works
        "Python Tutorial",
        "C++ Programming Guide",
    ];

    for title in &titles {
        db::bookmarks::insert_local(
            &mut tx,
            user.ap_user_id,
            InsertBookmark {
                url: format!(
                    "https://example.com/{}",
                    title.to_lowercase().replace(' ', "-")
                ),
                title: (*title).to_string(),
            },
            &app.base_url,
        )
        .await?;
    }

    tx.commit().await?;

    let home = app.req().get("/").await.test_page().await;
    let search_results = home
        .fill_form(
            "form[action='/search']",
            &SearchQuery {
                q: "Rust".to_string(),
                after_bookmark_id: None,
            },
        )
        .await
        .test_page()
        .await;

    // Test exact word match - searching for "Rust" should find 4 out of 6 Rust
    // bookmarks on first page (ordered by UUID)
    let html = search_results.dom.htmls();

    // Count how many Rust bookmarks appear in the first page
    let rust_count = titles.iter().filter(|&title| html.contains(title)).count();
    assert_eq!(
        rust_count, 4,
        "First page should show exactly 4 Rust bookmarks"
    );

    // Verify non-Rust bookmarks are not included
    assert!(!html.contains("Python Tutorial"));
    assert!(!html.contains("C++ Programming Guide"));

    // Test case insensitivity
    let search_results = app.req().get("/search?q=python").await.test_page().await;
    let html = search_results.dom.htmls();
    assert!(html.contains("Python Tutorial"));

    // Test partial word match - "gram" appears in "Programming" and "Guide"
    let search_results = app.req().get("/search?q=gram").await.test_page().await;
    let html = search_results.dom.htmls();
    // At least one bookmark with "Programming" or "Guide" should appear
    let has_programming = html.contains("Programming");
    let has_guide = html.contains("Guide");
    assert!(
        has_programming || has_guide,
        "Should find bookmarks containing 'gram'"
    );

    // Test special characters
    let search_results = app.req().get("/search?q=C%2B%2B").await.test_page().await;
    let html = search_results.dom.htmls();
    assert!(html.contains("C++ Programming Guide"));
    assert!(!html.contains("Python Tutorial"));

    Ok(())
}

#[test_log::test(tokio::test)]
async fn search_only_returns_users_own_bookmarks() -> anyhow::Result<()> {
    let mut app = TestApp::new().await;
    let user1 = app.create_test_user().await;
    let user2 = app.create_user("otheruser", "otherpassword").await;

    // Create bookmarks for both users with similar titles
    let mut tx = app.tx().await;
    let bookmark_1 = db::bookmarks::insert_local(
        &mut tx,
        user1.ap_user_id,
        InsertBookmark {
            url: "https://example.com/user1".to_string(),
            title: "My Rust Tutorial".to_string(),
        },
        &app.base_url,
    )
    .await?;
    let bookmark_2 = db::bookmarks::insert_local(
        &mut tx,
        user2.ap_user_id,
        InsertBookmark {
            url: "https://example.com/user2".to_string(),
            title: "Other User's Rust Guide".to_string(),
        },
        &app.base_url,
    )
    .await?;
    tx.commit().await?;

    let query = "/search?q=Rust";
    // Login as user1 and search
    app.login_test_user().await;
    let search_results = app.req().get(query).await.test_page().await;

    let html = search_results.dom.htmls();
    assert!(html.contains(&bookmark_1.id.to_string()));
    assert!(!html.contains(&bookmark_2.id.to_string()));

    // verify that the same query matches the other bookmark as well
    app.login_user(&user2.username, "otherpassword").await;
    let search_results = app.req().get(query).await.test_page().await;
    let html = search_results.dom.htmls();
    tracing::debug!("{}", search_results.dom.find("main").html());
    assert!(!html.contains(&bookmark_1.id.to_string()));
    assert!(html.contains(&bookmark_2.id.to_string()));

    Ok(())
}

#[test_log::test(tokio::test)]
async fn search_pagination_navigation() -> anyhow::Result<()> {
    // TODO update this test to use the pagination links provided in the html,
    // instead of generating the URLs inline here
    let mut app = TestApp::new().await;
    let user = app.create_test_user().await;
    app.login_test_user().await;

    // Create enough bookmarks to span multiple pages (page size is 4)
    let mut tx = app.tx().await;
    let mut bookmarks = Vec::new();
    for i in 1..=15 {
        let bookmark = db::bookmarks::insert_local(
            &mut tx,
            user.ap_user_id,
            InsertBookmark {
                url: format!("https://example.com/test{i}"),
                title: format!("Test Bookmark {i:02}"),
            },
            &app.base_url,
        )
        .await?;
        bookmarks.push((bookmark.id, bookmark.title.clone()));
    }
    tx.commit().await?;

    // Sort bookmarks by ID to match the database sort order
    bookmarks.sort_by_key(|(id, _)| *id);
    tracing::debug!("{bookmarks:#?}");

    // Test first page - should show first 4 bookmarks sorted by ID
    let first_page = app.req().get("/search?q=Test").await.test_page().await;
    for link in first_page.dom.find("a") {
        println!("- {}", link.outer_html());
    }
    let html = first_page.dom.find("main").htmls();
    assert!(html.contains(&bookmarks[0].1)); // First bookmark
    assert!(html.contains(&bookmarks[3].1)); // Fourth bookmark
    assert!(!html.contains(&bookmarks[4].1)); // Fifth bookmark shouldn't be visible
    assert!(html.contains("Next page"));
    assert!(!html.contains("Previous page"));

    // Test second page (forward pagination)
    let second_page = first_page.visit_link("Next page").await;
    for link in second_page.dom.find("a") {
        println!("- {}", link.outer_html());
    }
    let second_page_html = second_page.dom.find("main").htmls();
    assert!(second_page_html.contains(&bookmarks[4].1)); // Fifth bookmark
    assert!(second_page_html.contains(&bookmarks[7].1)); // Eighth bookmark
    assert!(!second_page_html.contains(&bookmarks[3].1)); // Fourth bookmark from page 1
    assert!(!second_page_html.contains(&bookmarks[8].1)); // Ninth bookmark from page 3
    assert!(second_page_html.contains("Previous page"));
    assert!(second_page_html.contains("Next page"));

    let third_page = second_page.visit_link("Next page").await;
    for link in third_page.dom.find("a") {
        println!("- {}", link.outer_html());
    }

    // Test backward pagination - go back to second page
    let back_to_second = third_page.visit_link("Previous page").await;
    for link in back_to_second.dom.find("a") {
        println!("- {}", link.outer_html());
    }

    let html = back_to_second.dom.find("main").htmls();
    assert!(html.contains(&bookmarks[4].1)); // Fifth bookmark
    assert!(html.contains(&bookmarks[7].1)); // Eighth bookmark
    assert!(!html.contains(&bookmarks[3].1)); // Fourth bookmark from page 1
    assert!(!html.contains(&bookmarks[8].1)); // Ninth bookmark from page 3
    assert_eq!(back_to_second.dom.find("main").htmls(), second_page_html);

    Ok(())
}

#[test_log::test(tokio::test)]
async fn search_preserves_query_in_pagination() -> anyhow::Result<()> {
    let mut app = TestApp::new().await;
    app.create_test_user().await;
    app.login_test_user().await;

    let search_results = app.req().get("/search?q=Rust").await.test_page().await;

    // Check that the search query is preserved in the pagination form
    assert!(search_results.dom.html().contains(r#"value="Rust""#));

    Ok(())
}

#[test_log::test(tokio::test)]
async fn search_requires_authentication() -> anyhow::Result<()> {
    let mut app = TestApp::new().await;

    // Try to search without logging in - should redirect to login page
    app.req()
        .expect_status(axum::http::StatusCode::SEE_OTHER)
        .get("/search?q=test")
        .await;

    Ok(())
}
