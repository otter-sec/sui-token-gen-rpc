//! # Token Generation Service Handlers
//!
//! This file contains the HTTP handler implementations for the Token Generation Service.
//! It provides endpoints for creating tokens, verifying repository URLs, and verifying content.
//! Each handler interacts with the `TokenServer` service for processing requests and returning responses.

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use service::TokenGen;
use std::sync::Arc;
use tarpc::context;

use crate::utils::{
    server_types::{ContentVerifyRequest, UrlVerifyRequest, VerifyUrlResponse},
    variables::VERIFICATION_MESSAGE,
};

use super::AppState;

/// Handler for the URL verification endpoint.
///
/// Verifies a given repository URL by attempting to clone it and checking its contents. Returns a JSON response
/// indicating whether the verification was successful or failed.
///
/// # Arguments
/// - `State(state)`: The shared application state, which includes the `TokenServer`.
/// - `AxumJson(payload)`: The JSON payload containing the URL to verify.
///
/// # Returns
/// - `Response`: A JSON response indicating the verification result, including success or error details.
pub async fn verify_url_handler(
    State(state): State<Arc<AppState>>,
    AxumJson(payload): AxumJson<UrlVerifyRequest>,
) -> Response {
    state
        .server
        .clone()
        .verify_url(context::current(), payload.url)
        .await
        .map_or_else(
            |e| {
                let response = VerifyUrlResponse {
                    success: false, // Indicates failure of URL verification
                    message: "Verification failed".to_string(),
                    error: Some(e.to_string()), // Detailed error message
                };
                (axum::http::StatusCode::BAD_REQUEST, AxumJson(response)).into_response()
            },
            |_| {
                let response = VerifyUrlResponse {
                    success: true, // Indicates success of URL verification
                    message: VERIFICATION_MESSAGE.to_string(),
                    error: None,
                };
                (axum::http::StatusCode::OK, AxumJson(response)).into_response()
            },
        )
}

/// Handler for the content verification endpoint.
///
/// Verifies the provided content (e.g., smart contract content) to ensure it meets the expected criteria.
/// Returns a JSON response indicating whether the verification was successful or failed.
///
/// # Arguments
/// - `State(state)`: The shared application state, which includes the `TokenServer`.
/// - `AxumJson(payload)`: The JSON payload containing the content to verify.
///
/// # Returns
/// - `Response`: A JSON response indicating the verification result, including success or error details.
pub async fn verify_content_handler(
    State(state): State<Arc<AppState>>,
    AxumJson(payload): AxumJson<ContentVerifyRequest>,
) -> Response {
    state
        .server
        .clone()
        .verify_content(context::current(), payload.content)
        .await
        .map_or_else(
            |e| {
                let response = VerifyUrlResponse {
                    success: false, // Indicates failure of content verification
                    message: "Verification failed".to_string(),
                    error: Some(e.to_string()), // Detailed error message
                };
                (axum::http::StatusCode::BAD_REQUEST, AxumJson(response)).into_response()
            },
            |_| {
                let response = VerifyUrlResponse {
                    success: true, // Indicates success of content verification
                    message: VERIFICATION_MESSAGE.to_string(),
                    error: None,
                };
                (axum::http::StatusCode::OK, AxumJson(response)).into_response()
            },
        )
}
