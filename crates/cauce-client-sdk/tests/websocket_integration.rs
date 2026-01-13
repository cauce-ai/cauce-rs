//! Integration tests for WebSocketTransport using a mock WebSocket server.

use cauce_client_sdk::{
    ClientConfig, ClientError, ConnectionState, JsonRpcMessage, Transport, WebSocketTransport,
};
use cauce_core::JsonRpcRequest;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::protocol::Message;

/// Helper to start a mock WebSocket server that echoes messages back.
async fn start_echo_server() -> (SocketAddr, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    if let Ok((stream, _)) = accept_result {
                        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
                        let (mut write, mut read) = ws_stream.split();

                        // Echo messages back
                        while let Some(Ok(msg)) = read.next().await {
                            match msg {
                                Message::Text(text) => {
                                    if write.send(Message::Text(text)).await.is_err() {
                                        break;
                                    }
                                }
                                Message::Ping(data) => {
                                    if write.send(Message::Pong(data)).await.is_err() {
                                        break;
                                    }
                                }
                                Message::Close(_) => break,
                                _ => {}
                            }
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
    });

    (addr, shutdown_tx)
}

/// Helper to start a mock server that closes immediately after accepting.
async fn start_closing_server() -> (SocketAddr, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    if let Ok((stream, _)) = accept_result {
                        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
                        let (mut write, _) = ws_stream.split();
                        // Close immediately
                        let _ = write.close().await;
                    }
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
    });

    (addr, shutdown_tx)
}

fn make_config(addr: SocketAddr) -> ClientConfig {
    ClientConfig::builder(format!("ws://{}", addr), "test-client")
        .connect_timeout(Duration::from_secs(5))
        .keepalive_interval(Duration::from_secs(30))
        .build()
        .expect("valid config")
}

#[tokio::test]
async fn test_connect_and_disconnect() {
    let (addr, _shutdown) = start_echo_server().await;
    let config = make_config(addr);
    let mut transport = WebSocketTransport::new(config);

    // Initial state
    assert_eq!(transport.state(), ConnectionState::Disconnected);

    // Connect
    transport.connect().await.expect("connect should succeed");
    assert_eq!(transport.state(), ConnectionState::Connected);
    assert!(transport.is_connected());

    // Disconnect
    transport
        .disconnect()
        .await
        .expect("disconnect should succeed");
    assert_eq!(transport.state(), ConnectionState::Disconnected);
    assert!(!transport.is_connected());
}

#[tokio::test]
async fn test_send_and_receive_roundtrip() {
    let (addr, _shutdown) = start_echo_server().await;
    let config = make_config(addr);
    let mut transport = WebSocketTransport::new(config);

    transport.connect().await.expect("connect should succeed");

    // Send a request
    let request = JsonRpcRequest::new(1.into(), "test.method".to_string(), None);
    let message = JsonRpcMessage::Request(request.clone());

    transport.send(message).await.expect("send should succeed");

    // Receive the echoed message
    let received = transport
        .receive()
        .await
        .expect("receive should succeed")
        .expect("should receive a message");

    assert!(received.is_request());
    if let JsonRpcMessage::Request(req) = received {
        assert_eq!(req.method, "test.method");
    } else {
        panic!("Expected request");
    }

    transport.disconnect().await.expect("disconnect");
}

#[tokio::test]
async fn test_send_when_not_connected() {
    let config = ClientConfig::builder("ws://localhost:9999", "test-client")
        .build()
        .expect("valid config");
    let mut transport = WebSocketTransport::new(config);

    let request = JsonRpcRequest::new(1.into(), "test.method".to_string(), None);
    let message = JsonRpcMessage::Request(request);

    let result = transport.send(message).await;
    assert!(matches!(result, Err(ClientError::NotConnected)));
}

#[tokio::test]
async fn test_receive_when_not_connected() {
    let config = ClientConfig::builder("ws://localhost:9999", "test-client")
        .build()
        .expect("valid config");
    let mut transport = WebSocketTransport::new(config);

    let result = transport.receive().await;
    assert!(matches!(result, Err(ClientError::NotConnected)));
}

#[tokio::test]
async fn test_connection_timeout() {
    // Connect to a non-routable IP to trigger timeout
    let config = ClientConfig::builder("ws://10.255.255.1:9999", "test-client")
        .connect_timeout(Duration::from_millis(100))
        .build()
        .expect("valid config");

    let mut transport = WebSocketTransport::new(config);
    let result = transport.connect().await;

    assert!(matches!(
        result,
        Err(ClientError::ConnectionTimeout { .. })
    ));
    assert_eq!(transport.state(), ConnectionState::Disconnected);
}

#[tokio::test]
async fn test_server_close_handling() {
    let (addr, _shutdown) = start_closing_server().await;
    let config = make_config(addr);
    let mut transport = WebSocketTransport::new(config);

    transport.connect().await.expect("connect should succeed");

    // Server should close the connection, receive should return None
    let result = transport.receive().await.expect("receive should not error");
    assert!(result.is_none());
    assert_eq!(transport.state(), ConnectionState::Disconnected);
}

#[tokio::test]
async fn test_multiple_messages() {
    let (addr, _shutdown) = start_echo_server().await;
    let config = make_config(addr);
    let mut transport = WebSocketTransport::new(config);

    transport.connect().await.expect("connect should succeed");

    // Send multiple messages
    for i in 1..=5 {
        let request = JsonRpcRequest::new(i.into(), format!("method.{}", i), None);
        let message = JsonRpcMessage::Request(request);
        transport.send(message).await.expect("send should succeed");
    }

    // Receive all messages
    for i in 1..=5 {
        let received = transport
            .receive()
            .await
            .expect("receive should succeed")
            .expect("should receive a message");

        if let JsonRpcMessage::Request(req) = received {
            assert_eq!(req.method, format!("method.{}", i));
        } else {
            panic!("Expected request");
        }
    }

    transport.disconnect().await.expect("disconnect");
}

#[tokio::test]
async fn test_disconnect_when_already_disconnected() {
    let config = ClientConfig::builder("ws://localhost:9999", "test-client")
        .build()
        .expect("valid config");
    let mut transport = WebSocketTransport::new(config);

    // Should not error even when already disconnected
    transport
        .disconnect()
        .await
        .expect("disconnect should succeed");
    assert_eq!(transport.state(), ConnectionState::Disconnected);
}

#[tokio::test]
async fn test_reconnect_after_disconnect() {
    let (addr, _shutdown) = start_echo_server().await;
    let config = make_config(addr);
    let mut transport = WebSocketTransport::new(config);

    // First connection
    transport.connect().await.expect("connect should succeed");
    assert!(transport.is_connected());

    // Disconnect
    transport.disconnect().await.expect("disconnect");
    assert!(!transport.is_connected());

    // Reconnect
    transport.connect().await.expect("reconnect should succeed");
    assert!(transport.is_connected());

    // Verify it works
    let request = JsonRpcRequest::new(1.into(), "test".to_string(), None);
    transport
        .send(JsonRpcMessage::Request(request))
        .await
        .expect("send should succeed");

    let received = transport
        .receive()
        .await
        .expect("receive should succeed")
        .expect("should receive message");
    assert!(received.is_request());

    transport.disconnect().await.expect("final disconnect");
}
