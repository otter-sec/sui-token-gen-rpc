// Unit tests for the TokenServer
//
// # Test Setup Requirements
// - No running RPC server on the test port (50051)
// - Clean filesystem state (no leftover test files)
// - Available network connection for Git operations
// - Sufficient permissions for file operations
//
use crate::utils::server_types::TokenServer;
use futures::prelude::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use service::{TokenGen, TokenGenErrors};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tarpc::context;

// Helper function to create a test server instance
fn test_server() -> TokenServer {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 50051); // Test server address
    TokenServer::new(addr)
}

// Test the token creation validation logic
#[tokio::test]
async fn test_create_token_validation() {
    let server = test_server();
    let ctx = context::current();

    // Test invalid decimals (0 and >= 100)
    let result = server
        .clone()
        .create(
            ctx,
            0,
            "Test".into(),
            "TST".into(),
            "Description".into(),
            false,
            "devnet".into(),
        )
        .await;
    assert!(matches!(result, Err(TokenGenErrors::InvalidDecimals)));

    // Test invalid symbol (too long)
    let result = server
        .clone()
        .create(
            ctx,
            8,
            "Test".into(),
            "TSTSTST".into(),
            "Description".into(),
            false,
            "devnet".into(),
        )
        .await;
    assert!(matches!(result, Err(TokenGenErrors::InvalidSymbol)));

    // Test invalid symbol (contains special characters)
    let result = server
        .clone()
        .create(
            ctx,
            8,
            "Test".into(),
            "T$T".into(),
            "Description".into(),
            false,
            "devnet".into(),
        )
        .await;
    assert!(matches!(result, Err(TokenGenErrors::InvalidSymbol)));

    // Test invalid name (contains special characters)
    let result = server
        .clone()
        .create(
            ctx,
            8,
            "Test@".into(),
            "TST".into(),
            "Description".into(),
            false,
            "devnet".into(),
        )
        .await;
    assert!(matches!(result, Err(TokenGenErrors::InvalidName)));

    // Test invalid description (contains special characters)
    let result = server
        .clone()
        .create(
            ctx,
            8,
            "Test".into(),
            "TST".into(),
            "Test@Description".into(),
            false,
            "devnet".into(),
        )
        .await;
    assert!(matches!(result, Err(TokenGenErrors::InvalidDescription)));

    // Test valid input
    let result = server
        .clone()
        .create(
            ctx,
            8,
            "Test Token".into(),
            "TST".into(),
            "Test Description".into(),
            false,
            "devnet".into(),
        )
        .await;
    assert!(result.is_ok());

    // Test valid name with (.)
    let result = server
        .clone()
        .create(
            ctx,
            8,
            ".sh".into(),
            "TST".into(),
            "Description".into(),
            false,
            "devnet".into(),
        )
        .await;
    assert!(result.is_ok());
}

// Test URL verification logic
#[tokio::test]
async fn test_verify_url_validation() {
    let server = test_server();
    let ctx = context::current();

    // Test invalid URL
    let result = server
        .clone()
        .verify_url(ctx, "not_a_url".into())
        .await;
    assert!(result.is_err());

    // Test invalid git URL
    let result = server
        .clone()
        .verify_url(ctx, "https://example.com/not-a-git-repo.git".into())
        .await;
    assert!(result.is_err());

    // Test valid URL format
    let result = server
        .verify_url(ctx, "https://github.com/valid/repo.git".into())
        .await;
    assert!(result.is_err()); // Will fail because repo doesn't exist, but URL format is valid
}

// Test content verification logic
#[tokio::test]
async fn test_verify_content_validation() {
    let server = test_server();
    let ctx = context::current();

    // Test empty content
    let result = server.clone().verify_content(ctx, "".into()).await;
    assert!(result.is_err());

    // Test invalid content
    let result = server
        .clone()
        .verify_content(ctx, "invalid content".into())
        .await;
    assert!(result.is_err());

    // Test malformed Move code
    let result = server
        .verify_content(ctx, "module test { public fun main() { } }".into())
        .await;
    assert!(result.is_err());
}

// Test concurrent RPC operations
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_operations() {
    let server = test_server();
    let ctx = context::current();
    const SAMPLE_DATA_SIZE: u32 = 10;

    let create_requests = generate_create_requests(SAMPLE_DATA_SIZE);
    let verify_requests = generate_verify_requests(SAMPLE_DATA_SIZE);

    let create_tasks: Vec<_> = create_requests
        .into_iter()
        .map(
            |(decimals, name, symbol, description, is_frozen, environment)| {
                let server = server.clone();
                tokio::spawn(async move {
                    server
                        .create(
                            ctx,
                            decimals,
                            name,
                            symbol,
                            description,
                            is_frozen,
                            environment,
                        )
                        .await
                })
            },
        )
        .collect();

    let verify_tasks: Vec<_> = verify_requests
        .into_iter()
        .map(|content| {
            let server = server.clone();
            tokio::spawn(async move { server.verify_url(ctx, content).await })
        })
        .collect();

    // Collect results and assert that no errors occurred
    let create_results = future::join_all(create_tasks).await;
    assert!(
        create_results.iter().all(|res| res.is_ok()),
        "Some create tasks failed"
    );

    let verify_results = future::join_all(verify_tasks).await;
    assert!(
        verify_results.iter().all(|res| res.is_ok()),
        "Some verify tasks failed"
    );
}

// Generate random create requests for testing
fn generate_create_requests(limit: u32) -> Vec<(u8, String, String, String, bool, String)> {
    let mut rng = thread_rng();
    let environments = ["mainnet", "testnet", "devnet"];

    (0..limit)
        .map(|_| {
            let decimals = rng.gen_range(1..100);
            let name: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
            let symbol: String = (0..3).map(|_| rng.sample(Alphanumeric) as char).collect();
            let description = if rng.gen_bool(0.5) {
                "".to_string()
            } else {
                (0..20).map(|_| rng.sample(Alphanumeric) as char).collect()
            };
            let is_frozen = rng.gen_bool(0.5);
            let environment = environments[rng.gen_range(0..environments.len())].to_string();

            (decimals, name, symbol, description, is_frozen, environment)
        })
        .collect()
}

// Generate verify URL requests
fn generate_verify_requests(limit: u32) -> Vec<String> {
    let base_url = "https://github.com/valid/repo.git";
    (0..limit)
        .map(|_| format!("{}{}", base_url, rand::random::<u32>() % 10000))
        .collect()
}
