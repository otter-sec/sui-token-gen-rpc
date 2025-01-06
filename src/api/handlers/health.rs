
use axum::{
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use serde_json::json;

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
