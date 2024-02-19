use anyhow::Result;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::PgPool;

use crate::{db::AppTx, response_error::ResponseError};

pub mod qs_form;
pub struct Tx(pub AppTx);

#[async_trait]
impl<S> FromRequestParts<S> for Tx
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ResponseError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.begin().await?;

        Ok(Self(conn))
    }
}
