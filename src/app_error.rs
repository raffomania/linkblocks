use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

// TODO rename this to ErrorResponse / ResponseError
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unknown Error")]
    Anyhow(#[source] anyhow::Error),
    #[error("Not Found")]
    NotFound,
    #[error("Authentication Failed")]
    NotAuthenticated,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("{self:?}");
        let status = match self {
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::NotAuthenticated => StatusCode::UNAUTHORIZED,
        };
        (status, self.to_string()).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            other => Self::Anyhow(other.into()),
        }
    }
}
