use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use serde_json::json;
use std::sync::Arc;
use tarpc::context;

use crate::{
    server_types::{CreateRequest, VerifyRequest},
    TokenServer,
};

use super::super::ApiError;

pub struct AppState {
    pub server: TokenServer,
}

/// Root endpoint handler that returns service information
pub async fn root_handler() -> AxumJson<serde_json::Value> {
    AxumJson(json!({
        "version": "0.16.0",
        "endpoints": [
            "/create",
            "/verify_url",
            "/verify_content",
            "/"
        ],
        "environment": "devnet",
        "status": "ok"
    }))
}

/// Handler for token creation endpoint
pub async fn create_handler(
    State(state): State<Arc<AppState>>,
    AxumJson(payload): AxumJson<CreateRequest>,
) -> Result<AxumJson<serde_json::Value>, Response> {
    match state
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
    {
        Ok((token, move_toml, test_token)) => Ok(AxumJson(json!({
            "token": token,
            "move_toml": move_toml,
            "test_token": test_token
        }))),
        Err(e) => Err(ApiError(e).into_response()),
    }
}

/// Handler for URL verification endpoint
pub async fn verify_url_handler(
    State(state): State<Arc<AppState>>,
    AxumJson(payload): AxumJson<VerifyRequest>,
) -> Result<AxumJson<()>, Response> {
    match state
        .server
        .clone()
        .verify_url(context::current(), payload.content)
        .await
    {
        Ok(()) => Ok(AxumJson(())),
        Err(e) => Err(ApiError(e).into_response()),
    }
}

/// Handler for content verification endpoint
pub async fn verify_content_handler(
    State(state): State<Arc<AppState>>,
    AxumJson(payload): AxumJson<VerifyRequest>,
) -> Result<AxumJson<()>, Response> {
    match state
        .server
        .clone()
        .verify_content(context::current(), payload.content)
        .await
    {
        Ok(()) => Ok(AxumJson(())),
        Err(e) => Err(ApiError(e).into_response()),
    }
}
