//! Example Cauce Protocol Subscriber (Agent)
//!
//! This example demonstrates how to create a client that subscribes
//! to signals from a Cauce hub.
//!
//! # Running
//!
//! First, start the hub server:
//! ```bash
//! cargo run -p hub-server
//! ```
//!
//! Then run the subscriber:
//! ```bash
//! cargo run -p subscriber
//! ```
//!
//! Finally, run the publisher to see signals being received:
//! ```bash
//! cargo run -p publisher
//! ```

use cauce_client_sdk::{CauceClient, ClientConfig, ClientType};
use tokio::signal;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Cauce Subscriber example");

    // Configure the client as an Agent
    let config = ClientConfig::builder("ws://127.0.0.1:8080/cauce/v1/ws", "example-subscriber")
        .client_type(ClientType::Agent)
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

    // Subscribe to example signals
    let topics = &["signal.example.*"];
    info!("Subscribing to topics: {:?}", topics);

    let mut subscription = match client.subscribe(topics).await {
        Ok(sub) => {
            info!("Subscribed! Subscription ID: {}", sub.subscription_id());
            sub
        }
        Err(e) => {
            error!("Failed to subscribe: {}", e);
            return Err(e.into());
        }
    };

    info!("Waiting for signals. Press Ctrl+C to stop.");
    info!("Run 'cargo run -p publisher' in another terminal to send signals.");

    // Process signals
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Shutdown signal received");
                break;
            }
            signal_opt = subscription.next() => {
                match signal_opt {
                    Some(signal) => {
                        info!("Received signal:");
                        info!("  ID: {}", signal.id);
                        info!("  Topic: {}", signal.topic.as_str());
                        info!("  Source: {} / {}", signal.source.type_, signal.source.adapter_id);
                        info!("  Timestamp: {}", signal.timestamp);

                        // Log payload content
                        if let Ok(payload_str) = serde_json::to_string_pretty(&signal.payload.raw) {
                            info!("  Payload: {}", payload_str);
                        }

                        // Acknowledge the signal
                        match client.ack(subscription.subscription_id(), &[&signal.id]).await {
                            Ok(_) => {
                                info!("  Acknowledged!");
                            }
                            Err(e) => {
                                warn!("  Failed to acknowledge: {}", e);
                            }
                        }
                    }
                    None => {
                        // Channel closed, subscription ended
                        warn!("Subscription channel closed");
                        break;
                    }
                }
            }
        }
    }

    // Unsubscribe and disconnect
    info!("Unsubscribing...");
    if let Err(e) = client.unsubscribe(subscription.subscription_id()).await {
        warn!("Error during unsubscribe: {}", e);
    }

    info!("Disconnecting...");
    if let Err(e) = client.disconnect().await {
        warn!("Error during disconnect: {}", e);
    }

    info!("Subscriber stopped");
    Ok(())
}
