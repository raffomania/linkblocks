use axum::{Router, http::StatusCode};
use sqlx::{Pool, Postgres};

use crate::{
    forms::users::CreateUser,
    server::{AppState, app},
};

use super::request_builder::RequestBuilder;

const TEST_USER_USERNAME: &str = "testuser";
const TEST_USER_PASSWORD: &str = "testpassword";

pub struct TestApp {
    router: Router,
    pool: Pool<Postgres>,
    logged_in_cookie: Option<String>,
}

impl TestApp {
    pub async fn new(pool: Pool<Postgres>) -> Self {
        TestApp {
            router: app(AppState {
                pool: pool.clone(),
                base_url: String::new(),
                demo_mode: false,
                oidc_state: crate::oidc::State::NotConfigured,
            })
            .await
            .unwrap(),
            pool,
            logged_in_cookie: None,
        }
    }

    pub fn req(&mut self) -> RequestBuilder {
        let mut req = RequestBuilder::new(&self.router);
        if let Some(cookie) = &self.logged_in_cookie {
            req = req.header(axum::http::header::COOKIE, cookie);
        }
        req
    }

    /// Since there's no route for creating users yet, we're doing this via the
    /// DB for now.
    pub async fn create_user(&self, username: &str, password: &str) {
        let mut tx = self
            .pool
            .begin()
            .await
            .expect("Failed to create transaction");
        crate::db::users::create_user_if_not_exists(
            &mut tx,
            CreateUser {
                username: username.to_string(),
                password: password.to_string(),
            },
        )
        .await
        .expect("Failed to create new user");
        tx.commit().await.expect("Failed to commit transaction");
    }

    pub async fn create_test_user(&self) {
        self.create_user(TEST_USER_USERNAME, TEST_USER_PASSWORD)
            .await;
    }

    pub async fn login_test_user(&mut self) {
        self.login_user(TEST_USER_USERNAME, TEST_USER_PASSWORD)
            .await;
    }

    pub async fn login_user(&mut self, username: &str, password: &str) {
        let login_page = self.req().get("/login").await.test_page().await;

        let input = crate::forms::users::Login {
            credentials: crate::forms::users::Credentials {
                username: username.to_string(),
                password: password.to_string(),
            },
            previous_uri: None,
        };

        let login_response = login_page
            .expect_status(StatusCode::SEE_OTHER)
            .fill_form("form", &input)
            .await;

        let cookie = login_response.headers().get("Set-Cookie").unwrap();
        let cookie = cookie.to_str().unwrap().split_once(';').unwrap().0;
        assert!(!cookie.is_empty());

        self.logged_in_cookie = Some(cookie.to_string());
    }
}
