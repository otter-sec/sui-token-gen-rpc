use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::handlers::{create_handler, root_handler, verify_content_handler, verify_url_handler, AppState};
use crate::TokenServer;

/// Build the axum router with all REST API routes
pub fn build_router(server: TokenServer) -> Router {
    let state = Arc::new(AppState { server });
    Router::new()
        .route("/", get(root_handler))
        .route("/create", post(create_handler))
        .route("/verify_url", post(verify_url_handler))
        .route("/verify_content", post(verify_content_handler))
        .with_state(state)
}
