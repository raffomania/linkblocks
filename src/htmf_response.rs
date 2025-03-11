use anyhow::anyhow;
use axum::response::{Html, IntoResponse};

use crate::response_error::ResponseError;

pub struct HtmfResponse(pub htmf::element::Element);

impl IntoResponse for HtmfResponse {
    fn into_response(self) -> axum::response::Response {
        if cfg!(not(debug_assertions)) {
            return Html(self.0.to_html()).into_response();
        }

        let Ok(html) = self.0.to_html_pretty() else {
            return ResponseError::Anyhow(anyhow!("Failed to serialize htmf element"))
                .into_response();
        };

        Html(html).into_response()
    }
}
