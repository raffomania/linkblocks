use axum::Router;
use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;
use url::Url;

use crate::{
    federation,
    forms::users::CreateUser,
    server::{app, AppState},
};

use super::request_builder::RequestBuilder;

pub struct TestApp {
    pub router: Router,
    pub pool: Pool<Postgres>,
    pub base_url: Url,
    pub state: AppState,
}

impl TestApp {
    pub async fn new() -> Self {
        let pool = super::db::new_pool().await;
        let base_url =
            Url::parse("http://localhost:4040").expect("Failed to parse URL for test instance");
        let state = AppState {
            pool: pool.clone(),
            base_url: base_url.clone(),
            demo_mode: false,
            oidc_state: crate::oidc::State::NotConfigured,
            federation_config: federation::config::new_config(pool.clone(), base_url.clone())
                .await
                .unwrap(),
        };

        TestApp {
            router: app(state.clone()).await.unwrap(),
            pool,
            base_url,
            state,
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
            &self.base_url,
        )
        .await
        .expect("Failed to create new user");
        tx.commit().await.expect("Failed to commit transaction");
    }

    pub async fn serve(&self) {
        let listener = TcpListener::bind("localhost:4040").await.unwrap();
        let router = self.router.clone();

        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
    }
}
