// This file defines the `TokenServer` struct and its methods for handling token-related requests.
// The `TokenServer` allows for token creation and verification. It provides methods for validating input,
// generating token content, verifying URLs, and verifying token content. The server interacts with
// utility functions for token generation and verification.

// Imports for external libraries and local modules
use regex::Regex;
use serde::{Deserialize, Serialize};
use service::{TokenGen, TokenGenErrors};
use std::net::SocketAddr;
use tarpc::context;

use crate::utils::{generation, helpers::sanitize_name, verify_helper}; // Utility functions

/// Request structure for creating a token.
///
/// This struct contains the necessary fields to request the creation of a token,
/// including decimals, name, symbol, description, frozen status, and the environment
/// (e.g., "mainnet", "devnet"). It is used when sending requests to create a new token.
#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub decimals: u8,        // Number of decimal places for the token
    pub name: String,        // Name of the token
    pub symbol: String,      // Symbol of the token (e.g., "BTC")
    pub description: String, // Description of the token (optional)
    pub is_frozen: bool,     // Flag indicating if the token is frozen
    pub environment: String, // Environment (mainnet, devnet, etc.)
}

/// Response structure for a successful token creation.
///
/// This structure contains the generated token content, the `Move.toml` configuration file,
/// and the test token content used for verification purposes after successful token creation.
#[derive(Debug, Serialize)]
pub struct CreateResponse {
    pub token: String,      // The generated token content
    pub move_toml: String,  // The Move.toml configuration file content
    pub test_token: String, // The generated test token content
}

/// Request structure for verifying token content.
///
/// This structure contains the content to be verified against the expected contract during validation.
/// It is used when verifying the content of an existing token.
#[derive(Debug, Deserialize)]
pub struct ContentVerifyRequest {
    pub content: String, // The content of the token to be verified
}

/// Request structure for verifying token URL.
///
/// This structure contains the URL (e.g., a Git repository URL) for verifying the token's existence
/// and validity. It is used when the client requests URL-based verification.
#[derive(Debug, Deserialize)]
pub struct UrlVerifyRequest {
    pub url: String, // The URL to be verified
}

/// Response structure for verifying a token URL.
///
/// This structure provides the result of the URL verification, including success status,
/// a message, and an optional error message if verification fails.
#[derive(Serialize)]
pub struct VerifyUrlResponse {
    pub success: bool,         // Indicates if the URL verification was successful
    pub message: String,       // A message explaining the result of the URL verification
    pub error: Option<String>, // Optional error message if the verification failed
}

/// The `TokenServer` struct that handles token-related requests.
///
/// This server binds to a specific socket address and provides methods for creating and verifying
/// tokens. It serves as the main server for processing token creation and validation requests.
#[derive(Clone)]
pub struct TokenServer {
    addr: SocketAddr, // The server's socket address
}

impl TokenServer {
    /// Constructor for `TokenServer`.
    ///
    /// Takes a socket address as input and initializes a new instance of `TokenServer`.
    ///
    /// # Arguments
    /// * `addr` - The socket address for the server to bind to.
    ///
    /// # Returns
    /// * `TokenServer` - A new instance of the `TokenServer`.
    pub fn new(addr: SocketAddr) -> Self {
        TokenServer { addr }
    }

    /// Logs the server address asynchronously.
    ///
    /// This method logs the server's address when handling a request. It can be useful for debugging
    /// or monitoring the server's activity.
    pub async fn log_address(&self) {
        tracing::info!("Server address: {}", self.addr);
    }
}

/// Implementation of the `TokenGen` trait for `TokenServer`.
///
/// This trait provides the core functionality for creating and verifying tokens. It includes methods
/// for validating token creation parameters, generating token content, verifying URLs, and comparing
/// token content against expected contracts.
impl TokenGen for TokenServer {
    /// Handles token creation with input validation.
    ///
    /// This method performs validation on the input fields such as `decimals`, `name`, `symbol`, `description`,
    /// and `environment`. If all validations pass, it generates the token content, Move.toml configuration file,
    /// and test token content for the request.
    ///
    /// # Arguments
    /// * `decimals` - The number of decimal places for the token.
    /// * `name` - The name of the token.
    /// * `symbol` - The symbol for the token.
    /// * `description` - A description of the token (optional).
    /// * `is_frozen` - Flag indicating if the token is frozen.
    /// * `environment` - The environment in which the token will be deployed (e.g., "mainnet", "devnet").
    ///
    /// # Returns
    /// * `Result<(String, String, String), TokenGenErrors>` - A result containing the generated token content,
    ///    Move.toml configuration file, and test token content, or an error if validation fails.
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
        // Log the server address when handling a request.
        self.log_address().await;

        // Validate decimals: must be between 1 and 99.
        if decimals == 0 || decimals >= 100 {
            return Err(TokenGenErrors::InvalidDecimals);
        }

        // Validate symbol: must be alphanumeric and no longer than 6 characters.
        let symbol_regex = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
        if !symbol_regex.is_match(&symbol) {
            return Err(TokenGenErrors::InvalidSymbol);
        }
        if symbol.len() > 6 {
            return Err(TokenGenErrors::InvalidSymbol);
        }

        // Validate name: must contain only alphanumeric characters, spaces, commas, or dots.
        let name_regex = Regex::new(r"^[a-zA-Z0-9\s,\.]+$").unwrap();
        if !name_regex.is_match(&name) {
            return Err(TokenGenErrors::InvalidName);
        }

        // Validate description: optional but must meet the same restrictions as the name.
        let description_valid_regex: Regex =
            Regex::new(r"^[a-zA-Z0-9\s.,'\!?;:(){}\[\]\-\_@#$%&*+=|~]+$").unwrap(); // Allows only alphanumeric, spaces and some special characters.
        if !description.is_empty() && !description_valid_regex.is_match(&description) {
            return Err(TokenGenErrors::InvalidDescription);
        }

        // Validate environment: must be one of "mainnet", "devnet", or "testnet".
        let valid_environments = ["mainnet", "devnet", "testnet"];
        let environment = if valid_environments.contains(&environment.as_str()) {
            environment
        } else {
            "devnet".to_string() // Default to "devnet" if invalid.
        };

        // Sanitize the name to create a valid folder name for storing the token.
        let base_folder: String = sanitize_name(&name);

        // Generate the token content and test token content using utility functions.
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

        // Generate the Move.toml configuration file for the token.
        let move_toml_content = generation::generate_move_toml(base_folder, environment);

        // Return the generated content as the response.
        Ok((token_content, move_toml_content, test_token_content))
    }

    /// Verifies the validity of a token URL.
    ///
    /// This method uses a helper function to verify the token at the specified URL.
    /// If the URL is invalid or the token cannot be verified, an error is returned.
    ///
    /// # Arguments
    /// * `url` - The URL (e.g., a Git repository URL) to be verified.
    ///
    /// # Returns
    /// * `Result<(), TokenGenErrors>` - A result indicating whether the URL verification was successful or not.
    async fn verify_url(
        self,
        _: context::Context,
        url: String,
    ) -> anyhow::Result<(), TokenGenErrors> {
        verify_helper::verify_token_using_url(&url)
            .await
            .map_err(|e| TokenGenErrors::VerifyResultError(e.to_string()))
    }

    /// Verifies the content of a token.
    ///
    /// This method compares the provided content against the expected contract content
    /// using a helper function. If the content does not match the expected contract,
    /// an error is returned.
    ///
    /// # Arguments
    /// * `content` - The content of the token to be verified.
    ///
    /// # Returns
    /// * `Result<(), TokenGenErrors>` - A result indicating whether the content verification was successful or not.
    async fn verify_content(
        self,
        _: context::Context,
        content: String,
    ) -> anyhow::Result<(), TokenGenErrors> {
        verify_helper::compare_contract_content(content)
            .map_err(|e| TokenGenErrors::VerifyResultError(e.to_string()))
    }

    /// Checks the health of the `TokenServer`.
    ///
    /// This method verifies that the server is operational and that critical components,
    async fn health_check(self, _: context::Context) -> anyhow::Result<(), TokenGenErrors> {
        // Log server health check status
        tracing::info!("Health check passed for server at {}", self.addr);
        Ok(())
    }
}
