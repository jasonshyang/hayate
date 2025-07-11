use clients::{BybitClient, BybitMessage};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create an unbounded channel for updates
    let (update_sender, _update_receiver) = mpsc::unbounded_channel::<BybitMessage>();

    // Create shutdown token
    let shutdown = tokio_util::sync::CancellationToken::new();

    // Create the Bybit client
    let mut client = BybitClient::new_with_shutdown(update_sender, shutdown.clone());

    // Connect to the Bybit WebSocket
    let handle = tokio::spawn(async move {
        if let Err(e) = client.connect().await {
            tracing::error!("Failed to connect to Bybit WebSocket: {}", e);
        }
    });

    // Wait for Ctrl+C to trigger shutdown
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    tracing::info!("Shutting down Bybit client...");
    shutdown.cancel();

    // Wait for the client to finish
    if let Err(e) = handle.await {
        tracing::error!("Error while waiting for Bybit client: {}", e);
    } else {
        tracing::info!("Bybit client shutdown successfully.");
    }
}
