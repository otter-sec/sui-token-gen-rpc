use axum::{body::Body, http::Request};
use serde_json::json;
use std::{env, fs};
use tower::ServiceExt;

use crate::api::router::build_router;
use crate::utils::variables::DEFAULT_ENVIRONMENT;
use crate::TokenServer;

fn test_server() -> TokenServer {
    let addr = "127.0.0.1:5000".parse().unwrap(); // Test server address
    TokenServer::new(addr)
}

/// Test the `/create` endpoint for token creation
#[tokio::test]
async fn test_create_token_api() {
    let server = test_server();
    let router = build_router(server);

    // Test invalid decimals (0)
    let request_payload = json!({
        "decimals": 0,
        "name": "Test",
        "symbol": "TST",
        "description": "Description",
        "is_frozen": false,
        "environment": DEFAULT_ENVIRONMENT
    });
    let response = router
        .clone()
        .oneshot(
            Request::post("/create")
                .header("Content-Type", "application/json")
                .body(Body::from(request_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400); // Expect 400 for invalid decimals

    // Test valid input
    let request_payload = json!({
        "decimals": 8,
        "name": "Test Token",
        "symbol": "TST",
        "description": "Test Description",
        "is_frozen": false,
        "environment": DEFAULT_ENVIRONMENT
    });
    let response = router
        .clone()
        .oneshot(
            Request::post("/create")
                .header("Content-Type", "application/json")
                .body(Body::from(request_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200); // Expect 200 for valid input
}

/// Test the `/verify_url` endpoint for URL validation
#[tokio::test]
async fn test_verify_url_api() {
    let server = test_server();
    let router = build_router(server);

    // Test invalid URL
    let request_payload = json!({ "url": "not_a_url" });
    let response = router
        .clone()
        .oneshot(
            Request::post("/verify_url")
                .header("Content-Type", "application/json")
                .body(Body::from(request_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400); // Expect 400 for invalid URL

    // Test valid URL
    let request_payload = json!({ "url": "https://github.com/meumar-osec/test-sui-token" });
    let response = router
        .clone()
        .oneshot(
            Request::post("/verify_url")
                .header("Content-Type", "application/json")
                .body(Body::from(request_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200); // Expect 200 for valid URL
}

/// Test the `/verify_content` endpoint for content validation
#[tokio::test]
async fn test_verify_content_api() {
    let server = test_server();
    let router = build_router(server);

    // Test empty content
    let request_payload = json!({ "content": "", "toml": "" });
    let response = router
        .clone()
        .oneshot(
            Request::post("/verify_content")
                .header("Content-Type", "application/json")
                .body(Body::from(request_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400); // Expect 400 for empty content

    // Test invalid content
    let request_payload = json!({ "content": "invalid content", "toml": "" });
    let response = router
        .clone()
        .oneshot(
            Request::post("/verify_content")
                .header("Content-Type", "application/json")
                .body(Body::from(request_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    // Get the current directory path
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let templates_path = format!(
        "{}/src/tests/tokens/valid_token.move",
        current_dir.display()
    );

    let toml_path = format!("{}/src/tests/tokens/valid_toml.toml", current_dir.display());

    // Read content from the existing valid token file
    let valid_content =
        fs::read_to_string(templates_path).expect("Failed to read valid token file");

    let valid_toml_content = fs::read_to_string(toml_path).expect("Failed to read valid toml file");

    // Test valid content (Move code)
    let request_payload = json!({
        "content": valid_content,
        "toml": valid_toml_content
    });

    let response = router
        .clone()
        .oneshot(
            Request::post("/verify_content")
                .header("Content-Type", "application/json")
                .body(Body::from(request_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200); // Expect 200 for valid content
}

/// Test concurrent API operations
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_api_operations() {
    let server = test_server();
    let router = build_router(server);

    const SAMPLE_DATA_SIZE: u32 = 10;
    let create_payloads = generate_create_requests(SAMPLE_DATA_SIZE);
    let verify_payloads = generate_verify_requests(SAMPLE_DATA_SIZE);

    let create_tasks: Vec<_> = create_payloads
        .into_iter()
        .map(|payload| {
            let router = router.clone();
            tokio::spawn(async move {
                router
                    .oneshot(
                        Request::post("/create")
                            .header("Content-Type", "application/json")
                            .body(Body::from(payload.to_string()))
                            .unwrap(),
                    )
                    .await
            })
        })
        .collect();

    let verify_tasks: Vec<_> = verify_payloads
        .into_iter()
        .map(|payload| {
            let router = router.clone();
            tokio::spawn(async move {
                router
                    .oneshot(
                        Request::post("/verify_url")
                            .header("Content-Type", "application/json")
                            .body(Body::from(payload.to_string()))
                            .unwrap(),
                    )
                    .await
            })
        })
        .collect();

    // Collect results and assert no errors occurred
    let create_results = futures::future::join_all(create_tasks).await;
    assert!(
        create_results.iter().all(|res| res.is_ok()),
        "Some create tasks failed"
    );

    let verify_results = futures::future::join_all(verify_tasks).await;
    assert!(
        verify_results.iter().all(|res| res.is_ok()),
        "Some verify tasks failed"
    );
}

// Helper to generate create requests
fn generate_create_requests(limit: u32) -> Vec<serde_json::Value> {
    (0..limit)
        .map(|_| {
            json!({
                "decimals": 8,
                "name": "Test Token",
                "symbol": "TST",
                "description": "Test Description",
                "is_frozen": false,
                "environment": DEFAULT_ENVIRONMENT
            })
        })
        .collect()
}

// Helper to generate verify URL requests
fn generate_verify_requests(limit: u32) -> Vec<serde_json::Value> {
    (0..limit)
        .map(|_| json!({ "url": "https://github.com/valid/repo.git" }))
        .collect()
}
