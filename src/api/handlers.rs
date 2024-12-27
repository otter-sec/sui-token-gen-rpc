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
use serde_json::json;
use service::TokenGen;
use std::sync::Arc;
use tarpc::context;

use crate::utils::server_types::{
    ContentVerifyRequest, CreateRequest, TokenServer, UrlVerifyRequest, VerifyUrlResponse,
};

/// Struct representing the shared application state.
///
/// Contains the `TokenServer` instance, which is responsible for processing requests.
pub struct AppState {
    pub server: TokenServer,
}

/// Handler for the token creation endpoint.
///
/// Accepts a JSON payload to create a new token with details such as decimals, name, symbol,
/// description, freeze status, and environment. It returns a JSON response with the created token's
/// details, associated metadata, or an error response.
///
/// # Arguments
/// - `State(state)`: The shared application state, which includes the `TokenServer`.
/// - `AxumJson(payload)`: The JSON payload containing the token creation request.
///
/// # Returns
/// - `Ok`: A JSON response with the generated token, MOVE TOML metadata, and test token metadata.
/// - `Err`: A response with an error message if the creation fails.
pub async fn create_handler(
    State(state): State<Arc<AppState>>,
    AxumJson(payload): AxumJson<CreateRequest>,
) -> Response {
    state
        .server
        .clone()
        .create(
            context::current(),
            payload.decimals,
            payload.name,
            payload.symbol,
            payload.description,
            payload.is_frozen,
            payload.environment,
        )
        .await
        .map_or_else(
            |e| {
                let response = VerifyUrlResponse {
                    success: false, // Indicates failure of token creation
                    message: "Creation failed".to_string(),
                    error: Some(e.to_string()), // Detailed error message
                };
                (axum::http::StatusCode::BAD_REQUEST, AxumJson(response)).into_response()
            },
            |(token, move_toml, test_token)| {
                // Successful token creation
                let response = json!({
                    "success": true,                  // Success flag
                    "message": "Creation successful".to_string(),
                    "data": {
                        "token": token,               // Generated token details
                        "move_toml": move_toml,       // MOVE TOML metadata
                        "test_token": test_token      // Test token metadata
                    }
                });
                (axum::http::StatusCode::OK, AxumJson(response)).into_response()
            },
        )
}

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
                    message: "Verified successfully".to_string(),
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
                    message: "Verified successfully".to_string(),
                    error: None,
                };
                (axum::http::StatusCode::OK, AxumJson(response)).into_response()
            },
        )
}

/// Handler for the health check endpoint.
///
/// This endpoint is used to check the health and availability of the service.
/// Returns a 200 OK status with a JSON response indicating the service is healthy.
///
/// # Returns
/// - `Response`: A JSON response with a success message.
pub async fn health_handler() -> Response {
    let response = json!({
        "success": true, // Indicates the service is healthy
        "message": "Service is up and running"
    });
    (axum::http::StatusCode::OK, AxumJson(response)).into_response()
}
