// This file handles incoming connections for both HTTP and RPC protocols.
// It accepts TCP connections, determines whether they are HTTP or RPC requests,
// and delegates the handling to the appropriate handler (HTTP handled by Axum,
// RPC handled by Tarpc). It also manages asynchronous tasks for efficient connection handling.

use super::*;
use crate::build_router;
use futures::prelude::*;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::TokioIo;
use service::TokenGen;
use std::net::SocketAddr;
use tarpc::{
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use tokio::net::TcpListener;

/// Helper function to detect if a request is an HTTP request.
///
/// This function checks the first few bytes of the incoming request to determine
/// if the request is likely an HTTP request. It recognizes common HTTP methods such as
/// GET, POST, PUT, HEAD, and DELETE.
///
/// # Arguments
/// * `buf` - The first few bytes of the incoming request.
///
/// # Returns
/// * `true` if the request matches a common HTTP method, `false` otherwise.
fn looks_like_http(buf: &[u8]) -> bool {
    // List of common HTTP methods (GET, POST, PUT, HEAD, DELETE)
    const HTTP_METHODS: [&[u8]; 5] = [
        b"GET ", b"POST", b"PUT ", b"HEAD", b"DELE", // First 4 bytes of DELETE
    ];

    // Check if the buffer starts with any HTTP method
    HTTP_METHODS.iter().any(|method| buf.starts_with(method))
}

/// Function to handle incoming connections asynchronously.
///
/// This function accepts TCP connections and spawns a task to handle each connection.
/// It distinguishes between HTTP and RPC requests by inspecting the first few bytes of
/// the incoming request. If the request is identified as an HTTP request, it delegates
/// the handling to the HTTP handler. If it is identified as an RPC request, it delegates
/// to the RPC handler.
///
/// # Arguments
/// * `listener` - The TCP listener that accepts incoming connections.
///
/// # Returns
/// * `Result<(), anyhow::Error>` indicating the success or failure of the operation.
pub async fn accept_connections(listener: TcpListener) -> anyhow::Result<()> {
    while let Ok((socket, peer_addr)) = listener.accept().await {
        tracing::info!("New connection from {}", peer_addr);

        // Spawns a new task to handle the connection asynchronously
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, peer_addr).await {
                tracing::error!("Error handling connection from {}: {}", peer_addr, e);
            }
        });
    }

    Ok(())
}

/// Handles individual connections (either HTTP or RPC).
///
/// This function first peeks at the initial bytes of the incoming request to determine
/// whether it is an HTTP or RPC request. If the request appears to be HTTP, it delegates
/// the handling to the `handle_http` function. If it appears to be an RPC request,
/// it delegates to the `handle_rpc` function.
///
/// # Arguments
/// * `socket` - The TCP socket for the incoming connection.
/// * `peer_addr` - The address of the peer making the request.
///
/// # Returns
/// * `Result<(), anyhow::Error>` indicating the success or failure of the operation.
async fn handle_connection(
    socket: tokio::net::TcpStream,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    let mut peek_buf = [0u8; 4];

    // Peek at the first 4 bytes of the request to detect the request type
    let n = socket.peek(&mut peek_buf).await?;
    if n < 4 {
        tracing::warn!("Received incomplete request header from {}", peer_addr);
        return Ok(()); // No further handling if the request is incomplete
    }

    // Check if the request looks like HTTP, and handle accordingly
    if looks_like_http(&peek_buf) {
        handle_http(socket, peer_addr).await?;
    } else {
        handle_rpc(socket, peer_addr).await?;
    }

    Ok(())
}

/// Handles HTTP connections using Axum (via Hyper).
///
/// This function creates a router with the server's endpoints and serves the HTTP connection
/// using Hyper's HTTP/1 server. The `TokenServer` instance is used to handle HTTP requests
/// routed through Axum.
///
/// # Arguments
/// * `socket` - The TCP socket for the incoming HTTP connection.
/// * `peer_addr` - The address of the peer making the request.
///
/// # Returns
/// * `Result<(), anyhow::Error>` indicating the success or failure of the operation.
async fn handle_http(socket: tokio::net::TcpStream, peer_addr: SocketAddr) -> anyhow::Result<()> {
    // Build the HTTP router for the TokenServer
    let router = build_router(TokenServer::new(peer_addr));
    let service = tower::ServiceBuilder::new().service(router);

    // Wrap the TCP stream for use with Hyper
    let io = TokioIo::new(socket);
    let service = service_fn(move |req| service.clone().oneshot(req));

    // Serve the HTTP connection with Hyper's HTTP/1 server
    http1::Builder::new()
        .serve_connection(io, service)
        .await
        .map_err(|e| anyhow::anyhow!("Error serving HTTP connection: {}", e))?;

    Ok(())
}

/// Handles RPC connections using Tarpc.
///
/// This function sets up a Tarpc server for RPC handling. It establishes a communication
/// channel over the incoming TCP stream and executes the Tarpc server asynchronously to
/// handle RPC requests.
///
/// # Arguments
/// * `socket` - The TCP socket for the incoming RPC connection.
/// * `peer_addr` - The address of the peer making the request.
///
/// # Returns
/// * `Result<(), anyhow::Error>` indicating the success or failure of the operation.
async fn handle_rpc(socket: tokio::net::TcpStream, peer_addr: SocketAddr) -> anyhow::Result<()> {
    // Create the TokenServer instance to handle RPC requests
    let server = TokenServer::new(peer_addr);

    // Set up the transport layer for the Tarpc server using JSON serialization
    let transport = tarpc::serde_transport::Transport::from((socket, Json::default()));

    // Create a base channel for communication between the client and the server
    let channel = server::BaseChannel::with_defaults(transport);

    // Execute the RPC server asynchronously
    channel.execute(server.serve()).for_each(spawn).await;

    Ok(())
}

/// Helper function to spawn asynchronous tasks.
///
/// This function allows async tasks to be executed concurrently, improving
/// efficiency by spawning them as independent tasks that can run in parallel.
///
/// # Arguments
/// * `fut` - The future representing the asynchronous task to be spawned.
///
/// # Returns
/// * `()` - The function returns nothing. It simply spawns the task.
async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
