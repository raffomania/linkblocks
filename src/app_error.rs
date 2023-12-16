use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Anyhow(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::debug!("{self:?}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
    }
}

impl<Error> From<Error> for AppError
where
    Error: Into<anyhow::Error>,
{
    fn from(value: Error) -> Self {
        Self::Anyhow(value.into())
    }
}
