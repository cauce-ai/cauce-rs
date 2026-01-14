//! Example Cauce Protocol Publisher (Adapter)
//!
//! This example demonstrates how to create a client that publishes
//! signals to a Cauce hub.
//!
//! # Running
//!
//! First, start the hub server:
//! ```bash
//! cargo run -p hub-server
//! ```
//!
//! Then run the publisher:
//! ```bash
//! cargo run -p publisher
//! ```

use cauce_client_sdk::{CauceClient, ClientConfig, ClientType};
use cauce_core::types::{Payload, Signal, Source, Topic};
use chrono::Utc;
use serde_json::json;
use std::time::Duration;
use tokio::signal;
use tokio::time::interval;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Cauce Publisher example");

    // Configure the client as an Adapter
    let config = ClientConfig::builder("ws://127.0.0.1:8080/cauce/v1/ws", "example-publisher")
        .client_type(ClientType::Adapter)
        .build()?;

    info!("Connecting to hub at ws://127.0.0.1:8080/cauce/v1/ws...");

    // Connect to the hub
    let mut client = match CauceClient::connect(config).await {
        Ok(client) => {
            info!("Connected! Session ID: {}", client.session_id().await.unwrap_or_default());
            client
        }
        Err(e) => {
            error!("Failed to connect: {}", e);
            error!("Make sure the hub server is running: cargo run -p hub-server");
            return Err(e.into());
        }
    };

    // Publish signals in a loop
    let mut ticker = interval(Duration::from_secs(2));
    let mut counter = 0u64;

    info!("Publishing signals every 2 seconds. Press Ctrl+C to stop.");

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Shutdown signal received");
                break;
            }
            _ = ticker.tick() => {
                counter += 1;

                // Create a sample signal
                let signal = create_sample_signal(counter);
                let topic = "signal.example.tick";

                info!("Publishing signal #{} to topic '{}'", counter, topic);

                match client.publish(topic, signal.into()).await {
                    Ok(response) => {
                        info!(
                            "  Published! message_id={}, delivered_to={}, queued_for={}",
                            response.message_id,
                            response.delivered_to,
                            response.queued_for
                        );
                    }
                    Err(e) => {
                        error!("  Failed to publish: {}", e);
                    }
                }
            }
        }
    }

    // Graceful disconnect
    info!("Disconnecting...");
    if let Err(e) = client.disconnect().await {
        error!("Error during disconnect: {}", e);
    }

    info!("Publisher stopped");
    Ok(())
}

/// Creates a sample signal with the given counter value
fn create_sample_signal(counter: u64) -> Signal {
    Signal {
        id: format!("sig_example_{}", counter),
        version: "1.0".to_string(),
        timestamp: Utc::now(),
        source: Source::new("example", "publisher-1", format!("tick-{}", counter)),
        topic: Topic::new_unchecked("signal.example.tick"),
        payload: Payload::new(
            json!({
                "counter": counter,
                "message": format!("Hello from publisher! This is tick #{}", counter),
                "timestamp": Utc::now().to_rfc3339()
            }),
            "application/json",
        ),
        metadata: None,
        encrypted: None,
    }
}
