//! # Axum Router Setup for Token Generation Service
//!
//! This file is responsible for setting up the Axum router with all the necessary HTTP routes for the Token Generation Service.
//! It configures the routes, sets up the shared state for the application, and applies middleware such as rate-limiting and error handling.
//!
//! The router is responsible for handling incoming HTTP requests and mapping them to the appropriate handlers, as well as applying
//! global rate-limiting to prevent excessive requests and ensure service reliability.

use super::handlers::{
    create_handler, health_handler, index, verify_address_handler, verify_content_handler,
    verify_url_handler, AppState,
};
use crate::utils::server::types::TokenServer;
use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, post},
    BoxError, Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
/// Builds and configures the Axum router for REST API routes.
///
/// This function sets up all the REST API routes for the Token Generation Service and configures the necessary middleware.
/// The router routes HTTP requests to the corresponding handler functions, and applies rate-limiting and error-handling layers.
/// The rate-limiting middleware helps prevent abuse of the API, and the error handler ensures that unhandled errors are
/// returned with an appropriate status code and message.
///
/// The following routes are defined:
/// - `/` [GET] - Root handler that returns basic service information and status.
/// - `/create` [POST] - Creates a new token by providing details such as name, symbol, etc.
/// - `/verify_url` [POST] - Verifies the validity of a repository URL by attempting to clone and check its content.
/// - `/verify_content` [POST] - Verifies the provided content (such as a smart contract) for correctness.
/// - `/health` [GET] - Returns a 200 OK status to indicate the service is healthy.
///
/// Additionally, the rate-limiting middleware is applied globally across all routes to ensure a maximum request rate.
///
/// # Arguments
/// * `server` - The `TokenServer` instance that handles the business logic and is shared across routes.
///
/// # Returns
/// A configured `Router` instance that contains all defined routes and middleware.
pub fn build_router(server: TokenServer) -> Router {
    // Create shared application state containing the TokenServer instance
    let state = Arc::new(AppState { server });

    // Define an error handler middleware to capture and format unhandled errors
    let error_handler = || {
        ServiceBuilder::new().layer(HandleErrorLayer::new(|err: BoxError| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled error: {}", err), // Format the error as a string for debugging
            )
        }))
    };

    // Rate-limiting middleware setup, configurable by the requests per second (req_per_sec)
    let global_rate_limit = |req_per_sec: u64| {
        ServiceBuilder::new()
            .layer(error_handler()) // Apply the error handler middleware
            .layer(BufferLayer::new(1024)) // Buffer incoming requests to avoid blocking
            .layer(RateLimitLayer::new(req_per_sec, Duration::from_secs(1))) // Apply rate-limiting: max `req_per_sec` requests per second
    };

    // Create the Axum router, add routes, and apply global middleware layers
    Router::new()
        .route("/", get(|| async { index() })) // Route for the root endpoint to check the service status
        .route("/create", post(create_handler)) // Route for token creation
        .route("/verify_url", post(verify_url_handler)) // Route for verifying URLs
        .route("/verify_content", post(verify_content_handler)) // Route for verifying content
        .route("/verify_address", post(verify_address_handler)) // Route for verifying address
        .route("/health", get(health_handler)) // New health check endpoint
        .layer(global_rate_limit(10000)) // Apply global rate-limiting to all routes in the router (10000 requests per second)
        .with_state(state) // Attach shared application state (TokenServer) to the router for use in all routes
}
