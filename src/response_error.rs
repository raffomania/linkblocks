use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type ResponseResult<T> = std::result::Result<T, ResponseError>;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Unknown Error")]
    Anyhow(#[source] anyhow::Error),
    #[error("Not Found")]
    NotFound,
    #[error("Authentication Failed")]
    NotAuthenticated,
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        tracing::error!("{self:?}");
        let status = match self {
            ResponseError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseError::NotFound => StatusCode::NOT_FOUND,
            // TODO redirect to login instead of sending an error
            ResponseError::NotAuthenticated => StatusCode::UNAUTHORIZED,
        };
        (status, self.to_string()).into_response()
    }
}

impl From<anyhow::Error> for ResponseError {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}

impl From<sqlx::Error> for ResponseError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            other => Self::Anyhow(other.into()),
        }
    }
}
