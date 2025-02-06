//! This is lifted from activitypub-federation as they
//! are not on axum 0.8 yet

use std::sync::LazyLock;

use activitypub_federation::FEDERATION_CONTENT_TYPE;
use axum::{
    http::{header, HeaderValue},
    response::IntoResponse,
};
use serde::Serialize;

/// Wrapper struct to respond with `application/activity+json` in axum handlers
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonResponse<Json: Serialize>(pub Json);

static SAFE_FEDERATION_CONTENT_TYPE: LazyLock<HeaderValue> = LazyLock::new(|| {
    #[expect(clippy::expect_used)]
    FEDERATION_CONTENT_TYPE
        .parse()
        .expect("Parsing 'application/activity+json' should never fail")
});

impl<Json: Serialize> IntoResponse for JsonResponse<Json> {
    fn into_response(self) -> axum::response::Response {
        let mut response = axum::response::Json(self.0).into_response();
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, SAFE_FEDERATION_CONTENT_TYPE.clone());
        response
    }
}
