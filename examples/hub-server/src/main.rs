//! Example Cauce Protocol Hub Server
//!
//! This example demonstrates how to create a simple Cauce hub server
//! that clients can connect to via WebSocket.
//!
//! # Running
//!
//! ```bash
//! cargo run -p hub-server
//! ```
//!
//! The server will start on `127.0.0.1:8080` and accept WebSocket connections
//! at `ws://127.0.0.1:8080/cauce/v1/ws`.

use cauce_server_sdk::{DefaultCauceServer, ServerConfig};
use std::net::SocketAddr;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Cauce Hub Server example");

    // Configure the server
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let config = ServerConfig::builder(addr)
        .server_name("example-hub")
        .build()?;

    info!("Server configured:");
    info!("  Address: {}", config.address);
    info!("  WebSocket: ws://{}/cauce/v1/ws", config.address);
    info!("  Health: http://{}/health", config.address);

    // Create the server with default in-memory components
    let server = DefaultCauceServer::new(config);

    // Start the server with graceful shutdown on Ctrl+C
    info!("Press Ctrl+C to stop the server");
    server.serve_with_shutdown(shutdown_signal()).await?;

    info!("Server stopped");
    Ok(())
}

/// Creates a future that completes when Ctrl+C is pressed
async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    info!("Shutdown signal received");
}
