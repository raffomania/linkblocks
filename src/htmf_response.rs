use anyhow::anyhow;
use axum::response::{Html, IntoResponse};

use crate::response_error::ResponseError;

pub struct HtmfResponse(htmf::element::Element);

impl IntoResponse for HtmfResponse {
    #[cfg(debug_assertions)]
    fn into_response(self) -> axum::response::Response {
        let maybe_html = self.0.to_html_pretty();
        if let Ok(html) = maybe_html {
            Html(html).into_response()
        } else {
            ResponseError::Anyhow(anyhow!("Failed to serialize htmf element")).into_response()
        }
    }

    #[cfg(not(debug_assertions))]
    fn into_response(self) -> axum::response::Response {
        Html(self.0.to_html()).into_response()
    }
}
