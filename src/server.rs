//! RPC and REST API Server Implementation for Sui Token Generator
//!
//! This module implements the server-side functionality for both RPC and REST API services:
//! - Token generation and validation service for both RPC and REST clients
//! - Server configuration, startup, and graceful shutdown
//! - Request handling and response processing for HTTP and RPC requests
//! - Concurrent request processing for high availability and performance
//! - REST API endpoints for token generation and validation
//! - RPC service for Sui token-related operations using Tarpc
//! - Helper functions for token generation, validation, and request handling
//! - Graceful shutdown and error handling mechanisms
//! - Test suite with comprehensive test cases to ensure functionality and reliability

use api::router::build_router;
use clap::Parser;
use service::init_tracing;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tower::ServiceExt;

mod server_helper;
// Module imports for modular structure
mod api;
#[cfg(test)]
mod tests;
mod utils;

use utils::server_types::*;

// Define command-line flags for server configuration
#[derive(Parser)]
struct Flags {
    #[clap(long)] // Specify the port for the server
    port: u16,
}

// Main function to initialize and run the RPC server
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command-line arguments for configuration
    let flags = Flags::parse();

    // Initialize tracing for diagnostics and monitoring
    init_tracing("Sui-token-get rpc")?;

    // Define the server address with the specified port
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), flags.port);

    // Create a TCP listener to accept incoming connections
    let listener = TcpListener::bind(&server_addr).await?;
    tracing::info!("Listening on port {}", listener.local_addr()?.port());

    // Graceful shutdown signal (e.g., Ctrl+C)
    let shutdown_signal = tokio::signal::ctrl_c();

    // Event-driven handling of incoming connections and shutdown
    // Uses tokio::select to handle either incoming connections or the shutdown signal
    tokio::select! {
        result = server_helper::accept_connections(listener) => {
            result.map_err(|e| anyhow::anyhow!("Error while accepting connections: {}", e))?;
        },
        _ = shutdown_signal => {
            tracing::info!("Shutdown signal received. Terminating server...");
        },
    }

    Ok(())
}
