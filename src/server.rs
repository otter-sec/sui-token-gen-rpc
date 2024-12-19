//! RPC Server Implementation for Sui Token Generator
//!
//! This module implements the server-side functionality:
//! - Token generation and validation service
//! - Server configuration and startup
//! - Request handling and response processing
//! - Concurrent request processing
//! - Test suite with comprehensive test cases
//! - Helper functions for token generation and validation

use clap::Parser;
use futures::{future, prelude::*};
use regex::Regex;
use service::{init_tracing, TokenGen, TokenGenErrors};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tarpc::{
    context,
    server::{self, Channel},
    tokio_serde::formats::Json,
};

// Module imports
mod utils;
use utils::{generation, helpers::sanitize_name, verify_helper}; // Utility functions

// Define command-line flags structure
#[derive(Parser)]
struct Flags {
    #[clap(long)] // Define the port flag
    port: u16,
}

// Define the TokenServer struct
#[derive(Clone)]
struct TokenServer {
    addr: SocketAddr, // The address for the server
}

impl TokenServer {
    // Constructor for TokenServer
    fn new(addr: SocketAddr) -> Self {
        TokenServer { addr }
    }

    // Logs the server address asynchronously
    async fn log_address(&self) {
        tracing::info!("Server address: {}", self.addr);
    }
}

// Implement the TokenGen trait for TokenServer
impl TokenGen for TokenServer {
    // Token creation logic with validation
    async fn create(
        self,
        _: context::Context,
        decimals: u8,
        name: String,
        symbol: String,
        description: String,
        is_frozen: bool,
        environment: String,
    ) -> anyhow::Result<(String, String, String), TokenGenErrors> {
        // Log the server address when handling the request
        self.log_address().await;

        // Validate decimals (should be between 1 and 99)
        if decimals == 0 || decimals >= 100 {
            return Err(TokenGenErrors::InvalidDecimals);
        }

        // Validate symbol: must be alphanumeric and no longer than 6 characters
        let symbol_regex = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
        if !symbol_regex.is_match(&symbol) {
            return Err(TokenGenErrors::InvalidSymbol);
        }
        if symbol.len() > 6 {
            return Err(TokenGenErrors::InvalidSymbol);
        }

        // Validate name: alphanumeric and allowed characters (spaces, commas, dots)
        let name_regex = Regex::new(r"^[a-zA-Z0-9\s,\.]+$").unwrap();
        if !name_regex.is_match(&name) {
            return Err(TokenGenErrors::InvalidName);
        }

        // Validate description (optional, but must meet same character restrictions as name)
        if !description.is_empty() && !name_regex.is_match(&description) {
            return Err(TokenGenErrors::InvalidDescription);
        }

        // Validate environment (must be one of "mainnet", "devnet", or "testnet")
        let valid_environments = ["mainnet", "devnet", "testnet"];
        let environment = if valid_environments.contains(&environment.as_str()) {
            environment
        } else {
            "devnet".to_string() // Default to "devnet" if invalid
        };

        // Proceed with token generation using utility functions
        let base_folder: String = sanitize_name(&name);
        let token_content: String = generation::generate_token(
            decimals,
            symbol.clone(),
            name.clone(),
            description.clone(),
            is_frozen,
            false,
        );
        let test_token_content: String = generation::generate_token(
            decimals,
            symbol.clone(),
            name.clone(),
            description.clone(),
            is_frozen,
            true,
        );

        let move_toml_content = generation::generate_move_toml(base_folder, environment);

        Ok((token_content, move_toml_content, test_token_content)) // Return the generated content
    }

    // Verify URL validity
    async fn verify_url(
        self,
        _: context::Context,
        url: String,
    ) -> anyhow::Result<(), TokenGenErrors> {
        verify_helper::verify_token_using_url(&url)
            .await
            .map_err(|e| TokenGenErrors::VerifyResultError(e.to_string()))
    }

    // Verify content validity
    async fn verify_content(
        self,
        _: context::Context,
        content: String,
    ) -> anyhow::Result<(), TokenGenErrors> {
        verify_helper::compare_contract_content(content)
            .map_err(|e| TokenGenErrors::VerifyResultError(e.to_string()))
    }
}

// Helper function to spawn async tasks
async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

// Main function to start the RPC server
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse(); // Parse command-line arguments
    init_tracing("Sui-token-get rpc")?; // Initialize tracing for logging
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), flags.port); // Set the server address
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?; // Start the listener
    tracing::info!("Listening on port {}", listener.local_addr().port());
    listener.config_mut().max_frame_length(10 * 1024 * 1024); // Set max frame size for RPC requests

    // Handle incoming requests asynchronously
    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = TokenServer::new(channel.transport().peer_addr().unwrap());
            channel.execute(server.serve()).for_each(spawn) // Spawn task for each channel
        })
        .buffer_unordered(1000) // Process requests concurrently
        .for_each(|_| async {})
        .await;

    Ok(())
}

// Unit tests for the TokenServer
//
// # Test Setup Requirements
// - No running RPC server on the test port (50051)
// - Clean filesystem state (no leftover test files)
// - Available network connection for Git operations
// - Sufficient permissions for file operations
//
#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::net::SocketAddr;
    use tarpc::context;

    // Helper function to create a test server instance
    fn test_server() -> TokenServer {
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 50051); // Test server address
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
                ctx.clone(),
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
                ctx.clone(),
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
                ctx.clone(),
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
                ctx.clone(),
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
                ctx.clone(),
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
                ctx.clone(),
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
            .verify_url(ctx.clone(), "not_a_url".into())
            .await;
        assert!(result.is_err());

        // Test invalid git URL
        let result = server
            .clone()
            .verify_url(ctx.clone(), "https://example.com/not-a-git-repo.git".into())
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
        let result = server.clone().verify_content(ctx.clone(), "".into()).await;
        assert!(result.is_err());

        // Test invalid content
        let result = server
            .clone()
            .verify_content(ctx.clone(), "invalid content".into())
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
                    let ctx = ctx.clone();
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
                let ctx = ctx.clone();
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
}
