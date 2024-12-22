pub mod rest;

use axum::response::{IntoResponse, Response};
use std::fmt;
use tarpc::client::RpcError;

use crate::TokenGenError;

/// API error type for converting TokenGenError to HTTP responses
#[derive(Debug)]
pub struct ApiError(pub TokenGenError);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<RpcError> for ApiError {
    fn from(err: RpcError) -> Self {
        ApiError(TokenGenError::RpcError(err))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            TokenGenError::InvalidDecimals(_) => axum::http::StatusCode::BAD_REQUEST,
            TokenGenError::InvalidSymbol(_) => axum::http::StatusCode::BAD_REQUEST,
            TokenGenError::InvalidName(_) => axum::http::StatusCode::BAD_REQUEST,
            TokenGenError::InvalidDescription(_) => axum::http::StatusCode::BAD_REQUEST,
            TokenGenError::InvalidEnvironment(_) => axum::http::StatusCode::BAD_REQUEST,
            TokenGenError::InvalidUrl(_) => axum::http::StatusCode::BAD_REQUEST,
            TokenGenError::InvalidContent(_) => axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            TokenGenError::RepositoryNotFound(_) => axum::http::StatusCode::NOT_FOUND,
            TokenGenError::RpcError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            TokenGenError::Other(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = serde_json::json!({
            "error": self.0.to_string()
        });

        (status, axum::Json(body)).into_response()
    }
}
