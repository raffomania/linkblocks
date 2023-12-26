use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Anyhow(anyhow::Error),
    NotFound(),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("{self:?}");
        match self {
            AppError::Anyhow(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
            AppError::NotFound() => (StatusCode::NOT_FOUND, "Not Found"),
        }
        .into_response()
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
            sqlx::Error::RowNotFound => Self::NotFound(),
            other => Self::Anyhow(other.into()),
        }
    }
}
