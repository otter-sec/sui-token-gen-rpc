use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use serde_json::json;
use std::sync::Arc;
use tarpc::context;

use crate::{CreateRequest, VerifyUrlResponse};

use super::AppState;

use service::TokenGen;
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