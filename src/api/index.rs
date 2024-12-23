use axum::Json;
use serde_json::{json, Value};
use std::sync::OnceLock;

/// This file defines the `index` handler for the API, which serves as the root endpoint
/// for providing API documentation and metadata about the service.
///
/// The `index` function returns a static JSON response containing metadata for the
/// API, such as the version, available API endpoints, and current environment status.
///
/// Static JSON response for the index endpoint.
/// The `OnceLock` ensures that the response is computed only once, and subsequent
/// calls will reuse the same response, improving efficiency.
static INDEX_JSON: OnceLock<Value> = OnceLock::new();

/// Handler for the index endpoint that provides API documentation and metadata about the service.
///
/// # Endpoint: GET /
///
/// This endpoint returns a JSON object that includes:
/// - The version of the service.
/// - A list of available API endpoints with their HTTP methods, expected request payloads, and response structures.
/// - The current environment (e.g., devnet, testnet, mainnet).
/// - The current service status (e.g., "ok").
///
/// # Returns
/// - `Json<Value>`: A JSON response containing API documentation, including details on endpoints and the service's current status.
///
/// # Example Response:
/// ```json
/// {
///   "version": "0.16.0",
///   "endpoints": [ ... ],
///   "environment": "devnet",
///   "status": "ok"
/// }
/// ```
pub fn index() -> Json<Value> {
    // Lazily initialize the static JSON response using `OnceLock` to ensure it is created only once
    let value = INDEX_JSON.get_or_init(|| {
        json!({
            "version": "0.16.0",                  // The current version of the service
            "endpoints": [                       // List of available endpoints with their HTTP methods
                {
                    "path": "/create",          // Endpoint path for creating a token
                    "method": "POST",           // HTTP method for the /create endpoint
                    "payload": {                // Expected input parameters for the /create endpoint
                        "decimals": "Number of decimals (integer)",
                        "name": "Token name (string)",
                        "symbol": "Token symbol (string)",
                        "description": "Token description (string)",
                        "is_frozen": "Token freeze status (boolean)",
                        "environment": "The environment (devnet, testnet, mainnet)"
                    },
                    "response": {               // Structure of the response returned from the /create endpoint
                        "success": "Indicates if token creation was successful (boolean)",
                        "message": "Message describing the outcome (string)",
                        "data": {                // Token creation data
                            "token": "Generated token details (string)",
                            "move_toml": "Metadata related to MOVE TOML (string)",
                            "test_token": "Test token metadata (string)"
                        }
                    }
                },
                {
                    "path": "/verify_url",      // Endpoint path for verifying a URL
                    "method": "POST",           // HTTP method for the /verify_url endpoint
                    "payload": {                // Expected input parameters for the /verify_url endpoint
                        "url": "Repository URL to verify (string)"
                    },
                    "response": {               // Structure of the response returned from the /verify_url endpoint
                        "success": "Indicates if URL verification was successful (boolean)",
                        "message": "Message describing the outcome (string)",
                        "error": "Error message if verification failed (string, optional)"
                    }
                },
                {
                    "path": "/verify_content",  // Endpoint path for verifying content
                    "method": "POST",           // HTTP method for the /verify_content endpoint
                    "payload": {                // Expected input parameters for the /verify_content endpoint
                        "content": "Content to verify (string)"
                    },
                    "response": {               // Structure of the response returned from the /verify_content endpoint
                        "success": "Indicates if content verification was successful (boolean)",
                        "message": "Message describing the outcome (string)",
                        "error": "Error message if verification failed (string, optional)"
                    }
                },
                {
                    "path": "/",                // Endpoint path for the root check
                    "method": "GET",            // HTTP method for the root endpoint
                    "response": {               // Structure of the response returned from the root endpoint
                        "version": "Service version (string)",
                        "endpoints": "List of available endpoints (array of strings)",
                        "environment": "Current environment (string)",
                        "status": "Service status (string)"
                    }
                }
            ],
            "environment": "devnet",             // Current environment (devnet, testnet, mainnet)
            "success": true                       // Current status of the service (ok, error, etc.)
        })
    });

    // Return the cached JSON response as a `Json<Value>` for the API documentation
    Json(value.clone())
}
