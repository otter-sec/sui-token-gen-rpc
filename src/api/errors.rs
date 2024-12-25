//! # API Error Handling for Token Generation Service
//!
//! This file defines error handling for the Token Generation Service. It converts application-specific
//! errors (defined in `TokenGenErrors`) into standardized HTTP error responses. This ensures that errors
//! are returned in a consistent format, with appropriate HTTP status codes, and are easily consumable by clients.
//!
//! The `ApiError` struct wraps the `TokenGenErrors` enum and implements the `IntoResponse` trait to convert
//! errors into structured HTTP responses that follow RESTful principles.

use axum::{
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use hyper::StatusCode;
use service::TokenGenErrors;
use std::fmt;

/// Wrapper type for `TokenGenErrors` to implement standardized HTTP error responses.
///
/// This struct wraps application-specific errors (such as `TokenGenErrors`) and provides a mechanism
/// to convert them into consistent HTTP responses. The conversion includes mapping the errors to appropriate
/// HTTP status codes and formatting them into a JSON structure for easy interpretation by the client.
pub struct ApiError(pub TokenGenErrors);

/// Implements a conversion from `TokenGenErrors` to `ApiError`.
///
/// This implementation enables seamless conversion when application-specific errors (`TokenGenErrors`) are encountered,
/// allowing for an easier flow when dealing with errors and improving readability by abstracting error details into a common API format.
impl From<TokenGenErrors> for ApiError {
    fn from(err: TokenGenErrors) -> Self {
        ApiError(err)
    }
}

/// Implements the `Display` trait for `ApiError`.
///
/// This provides a human-readable string representation of the error for easier debugging and logging.
/// The `Display` trait ensures that when an `ApiError` is printed or logged, it provides a clear message about the error.
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0) // Formats the inner `TokenGenErrors` enum variant as a string
    }
}

/// Implements the `IntoResponse` trait for `ApiError`.
///
/// This is the core of the error handling logic, converting `ApiError` into a structured HTTP response
/// that Axum can return. The response includes an appropriate HTTP status code, a human-readable error message,
/// and metadata in JSON format that can be used by the client to understand the nature of the error.
///
/// The `IntoResponse` trait is implemented so that `ApiError` can be directly used in Axum route handlers to
/// generate consistent error responses.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Determine the HTTP status code and error message based on the specific error type
        let (status, error_message) = match &self.0 {
            // Client-side errors (400 Bad Request)
            TokenGenErrors::InvalidDecimals
            | TokenGenErrors::InvalidSymbol
            | TokenGenErrors::InvalidName
            | TokenGenErrors::InvalidDescription => (StatusCode::BAD_REQUEST, self.to_string()),

            // Content validation errors (422 Unprocessable Entity)
            TokenGenErrors::ProgramModified
            | TokenGenErrors::ContractModified
            | TokenGenErrors::VerifyResultError(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }

            // Resource not found errors (404 Not Found)
            TokenGenErrors::ClonedRepoNotFound
            | TokenGenErrors::InvalidPath(_)
            | TokenGenErrors::InvalidUrl(_) => (StatusCode::NOT_FOUND, self.to_string()),

            // Server-side errors (500 Internal Server Error)
            TokenGenErrors::GitError(_)
            | TokenGenErrors::FileIoError(_)
            | TokenGenErrors::GeneralError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        // Create the JSON response that will be returned to the client
        (
            status,
            AxumJson(serde_json::json!({
                "error": error_message,           // A human-readable error message for debugging
                "code": status.as_u16(),          // The HTTP status code (e.g., 400, 422)
                "status": status.to_string()      // A string representation of the HTTP status code (e.g., "400 Bad Request")
            })),
        )
            .into_response() // Convert the tuple into an HTTP response
    }
}
