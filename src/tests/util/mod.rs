use axum::Router;
use sqlx::{Pool, Postgres};

use crate::server::app;

use self::request_builder::RequestBuilder;

pub mod request_builder;

pub struct TestApp {
    router: Router,
}

impl TestApp {
    pub async fn new(pool: Pool<Postgres>) -> Self {
        TestApp {
            router: app(pool).await.unwrap(),
        }
    }
    pub fn req(&mut self) -> RequestBuilder {
        RequestBuilder::new(&mut self.router)
    }
}
