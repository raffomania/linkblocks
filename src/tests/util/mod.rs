use axum::Router;
use sqlx::{Pool, Postgres};

use crate::server::{app, AppState};

use self::request_builder::RequestBuilder;

pub mod dom;
pub mod request_builder;

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
            })
            .await
            .unwrap(),
        }
    }
    pub fn req(&mut self) -> RequestBuilder {
        RequestBuilder::new(&mut self.router)
    }
}
