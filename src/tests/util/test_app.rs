use axum::Router;
use sqlx::{Pool, Postgres};

use crate::{
    forms::users::CreateUser,
    server::{app, AppState},
};

use super::request_builder::RequestBuilder;

pub struct TestApp {
    router: Router,
    pool: Pool<Postgres>,
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
        }
    }

    pub fn req(&mut self) -> RequestBuilder {
        RequestBuilder::new(&self.router)
    }

    /// Since there's no route for creating users yet, we're doing this via the DB for now.
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
}
