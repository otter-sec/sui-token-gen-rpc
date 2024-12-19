//! RPC Client Implementation for Sui Token Generator
//!
//! This module implements the client-side functionality:
//! - Command-line interface for token operations
//! - Client configuration and connection setup
//! - Error handling and validation
//! - RPC request handling and response processing
//! - Parameter validation and sanitization

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use tarpc::{client, context, service, tokio_serde::formats::Json};
use thiserror::Error;

// Allow the dead_code warning for the utils module, as some code may not be used in the client.
#[allow(dead_code)]
mod utils;
use utils::errors::TokenGenErrors; // Import custom error types for token generation

// Define a Tarpc service trait for token generation with methods to create tokens and verify URLs and content.
#[service]
pub trait TokenGen {
    // The `create` method accepts parameters necessary for creating a new token and returns a result containing
    // token information (string) or an error.
    #[allow(clippy::too_many_arguments)] // Suppress warning for too many parameters in function signature
    async fn create(
        decimals: u8,        // The number of decimals for the token
        name: String,        // The name of the token
        symbol: String,      // The symbol representing the token
        description: String, // Description of the token
        is_frozen: bool,     // Whether the token is frozen or not
        environment: String, // The environment (mainnet, devnet, testnet)
    ) -> Result<(String, String, String), TokenGenErrors>; // Return a tuple of generated token information or an error

    // Method to verify the URL provided for the token (e.g., GitHub URL)
    async fn verify_url(url: String) -> Result<(), TokenGenErrors>;

    // Method to verify the content of the token's code or configuration
    async fn verify_content(content: String) -> Result<(), TokenGenErrors>;
}

// Define a custom error enum for the client, handling various error scenarios.
#[derive(Debug, Error)]
enum ClientError {
    #[error("Missing required parameter: {0}")]
    MissingParameter(String), // Error for missing required parameters

    #[error("Failed to communicate with server: {0}")]
    ServerError(#[from] tarpc::client::RpcError), // Error for RPC communication failures

    #[error("Invalid inputs: {0}")]
    InvalidInputs(String), // Error for invalid input values
}

// Implement conversion from `TokenGenErrors` to `ClientError` for easier error handling.
impl From<TokenGenErrors> for ClientError {
    fn from(err: TokenGenErrors) -> Self {
        ClientError::InvalidInputs(err.to_string())
    }
}

// Define a struct to represent command-line flags using the Clap library.
#[derive(Parser)]
struct Flags {
    #[clap(long)] // Define flag for server address
    server_addr: SocketAddr,

    #[clap(long)] // Define flag for command to execute
    command: Option<String>,

    #[clap(long)] // Define flag for token name
    name: Option<String>,

    #[clap(long)] // Define flag for token symbol
    symbol: Option<String>,

    #[clap(long)] // Define flag for token description
    description: Option<String>,

    #[clap(long)] // Define flag for token decimals
    decimals: Option<u8>,

    #[clap(long)] // Define flag for frozen status of the token
    is_frozen: Option<bool>,

    #[clap(long)] // Define flag for environment setting
    environment: Option<String>,

    #[clap(long)] // Define flag for verifying URL
    url: Option<String>,

    #[clap(long)] // Define flag for verifying content
    content: Option<String>,
}

// Main asynchronous entry point for the client.
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line flags
    let flags = Flags::parse();

    // Establish a connection to the server using Tarpc and JSON serialization.
    let mut transport = tarpc::serde_transport::tcp::connect(flags.server_addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX); // Set max frame length for transport

    // Create a Tarpc client instance for the `TokenGen` service
    let client: TokenGenClient =
        TokenGenClient::new(client::Config::default(), transport.await?).spawn();

    // Handle the command based on user input flags
    if let Err(err) = handle_command(flags, client).await {
        // Print the error message if handling the command fails
        eprintln!("Error: {}", err);
    }

    Ok(())
}

// Helper function to ensure required parameters are present.
fn require_param<T>(param: Option<T>, param_name: &str, command: &str) -> Result<T, ClientError> {
    // Return the parameter if it exists, otherwise return a custom error
    param.ok_or_else(|| {
        ClientError::MissingParameter(format!(
            "{} is required for '{}' command",
            param_name, command
        ))
    })
}

// Function to handle the command based on the provided flags and interact with the Tarpc client.
async fn handle_command(flags: Flags, client: TokenGenClient) -> Result<(), ClientError> {
    // Match the command provided in the flags
    match flags.command.as_deref() {
        Some("verify_url") => {
            // Handle the verify_url command
            let url = require_param(flags.url, "url", "verify")?;
            let result = client.verify_url(context::current(), url).await?;
            println!("Verification Result: {:?}", result);
        }
        Some("verify_content") => {
            // Handle the verify_content command
            let content = require_param(flags.content, "content", "verify")?;
            let result = client.verify_content(context::current(), content).await?;
            println!("Verification Result: {:?}", result);
        }
        Some("create") => {
            // Handle the create command to generate a new token
            let name = require_param(flags.name, "name", "create")?;
            let symbol = require_param(flags.symbol, "symbol", "create")?;
            let description = require_param(flags.description, "description", "create")?;
            let decimals = require_param(flags.decimals, "decimals", "create")?;
            let is_frozen = require_param(flags.is_frozen, "is_frozen", "create")?;
            let environment = require_param(flags.environment, "environment", "create")?;

            // Call the TokenGen client's `create` method to generate the token
            let result = client
                .create(
                    context::current(),
                    decimals,
                    name,
                    symbol,
                    description,
                    is_frozen,
                    environment,
                )
                .await
                .map_err(ClientError::ServerError)?;

            // Print the generated token content
            let (token_content, move_toml_content, test_token_content) = result?;
            println!("Token Content: {}", token_content);
            println!("Move.toml Content: {}", move_toml_content);
            println!("Test token Content: {}", test_token_content);
        }
        Some(cmd) => return Err(ClientError::InvalidInputs(cmd.into())), // Handle invalid commands
        None => {
            // Handle the case where no command is provided
            return Err(ClientError::MissingParameter(
                "command is required (either 'create' or 'verify')".into(),
            ));
        }
    }

    Ok(())
}
