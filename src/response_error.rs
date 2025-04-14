use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type ResponseResult<T> = std::result::Result<T, ResponseError>;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Not Found")]
    NotFound,
    #[error("Authentication Failed")]
    NotAuthenticated,
    #[error("Unknown Error")]
    Anyhow(#[from] anyhow::Error),
    #[error("Internal Error")]
    Url(#[from] url::ParseError),
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        tracing::error!("{self:?}");
        let status = match self {
            ResponseError::NotFound => StatusCode::NOT_FOUND,
            // TODO redirect to login instead of returning an error
            ResponseError::NotAuthenticated => StatusCode::UNAUTHORIZED,
            ResponseError::Anyhow(_) | ResponseError::Url(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
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
