use axum::Router;
use sqlx::{Pool, Postgres};

use crate::server::{app, AppState};

use super::request_builder::RequestBuilder;

pub struct TestApp {
    router: Router,
}

impl TestApp {
    pub async fn new(pool: Pool<Postgres>) -> Self {
        TestApp {
            router: app(AppState {
                pool,
                base_url: String::new(),
                demo_mode: false,
                oidc_state: crate::oidc::State::NotConfigured,
            })
            .await
            .unwrap(),
        }
    }

    pub fn req(&mut self) -> RequestBuilder {
        RequestBuilder::new(&self.router)
    }
}
