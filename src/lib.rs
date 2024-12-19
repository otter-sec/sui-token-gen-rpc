//! RPC Service Implementation for Sui Token Generator
//!
//! This module provides the core RPC functionality including:
//! - Token generation service trait definition
//! - Error type definitions and handling
//! - OpenTelemetry tracing configuration
//! - Service-wide type definitions and constants

use anyhow::Result;
use opentelemetry::trace::TracerProvider as _;
use serde::{Deserialize, Serialize};
use tarpc::service;
use thiserror::Error;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

// Define a Tarpc service for token generation operations.
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

// Define a custom error enum to handle various errors related to token generation.
#[derive(Error, Debug, Deserialize, Serialize)]
pub enum TokenGenErrors {
    #[error("Given contract is modified")]
    ProgramModified, // Error for when the contract is modified unexpectedly

    #[error("Invalid decimals provided")]
    InvalidDecimals, // Error when the decimals provided are invalid

    #[error("Invalid symbol provided")]
    InvalidSymbol, // Error when the symbol provided is invalid

    #[error("Invalid name provided")]
    InvalidName, // Error when the name provided is invalid

    #[error("Invalid description provided")]
    InvalidDescription, // Error when the description provided is invalid

    #[error("Content mismatch detected")]
    ContractModified, // Error for when the content does not match expected contract content

    #[error("Cloned repo not found")]
    ClonedRepoNotFound, // Error when the cloned repository is not found

    #[error("An error occurred: {0}")]
    GeneralError(String), // A general error with a dynamic message

    #[error("Invalid path: {0}")]
    InvalidPath(String), // Error for invalid file paths

    #[error("Invalid URL: {0}")]
    InvalidUrl(String), // Error for invalid URLs

    #[error("Git operation failed: {0}")]
    GitError(String), // Error for issues with Git operations

    #[error("File I/O error: {0}")]
    FileIoError(String), // Error for issues with file input/output operations

    #[error("{0}")]
    VerifyResultError(String), // Error when verification of the token's result fails
}

// Function to initialize OpenTelemetry tracing for the service.
// This sets up the tracing configuration and exporter to send traces to an OTLP (OpenTelemetry Protocol) backend.
pub fn init_tracing(service_name: &'static str) -> anyhow::Result<()> {
    // Set up the OpenTelemetry pipeline with tracing, batch configuration, and exporter.
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing() // Configure tracing capabilities
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new([opentelemetry::KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                service_name, // Name of the service for which traces are collected
            )]),
        ))
        .with_batch_config(opentelemetry_sdk::trace::BatchConfig::default()) // Set batch configuration for traces
        .with_exporter(opentelemetry_otlp::new_exporter().tonic()) // Export traces via OTLP using tonic
        .install_batch(opentelemetry_sdk::runtime::Tokio)?; // Install the batch exporter using Tokio runtime

    // Set the global tracer provider
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    // Create a tracer from the tracer provider
    let tracer = tracer_provider.tracer(service_name);

    // Initialize the tracing subscriber with environment-based filtering, formatted logging, and OpenTelemetry integration.
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env()) // Use environment variable for logging level
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)) // Log span events for new and closed spans
        .with(tracing_opentelemetry::layer().with_tracer(tracer)) // Integrate OpenTelemetry with tracing
        .try_init()?; // Initialize the tracing subscriber

    Ok(()) // Return Ok to indicate successful initialization
}
