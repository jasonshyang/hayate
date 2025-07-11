use clients::{BybitClient, BybitMessage};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create an unbounded channel for updates
    let (update_sender, _update_receiver) = mpsc::unbounded_channel::<BybitMessage>();

    // Create the Bybit client
    let mut client = BybitClient::new(update_sender);

    // Connect to the Bybit WebSocket
    tokio::select! {
        result = client.connect() => {
            match result {
                Ok(_) => tracing::info!("Connected to Bybit WebSocket"),
                Err(e) => tracing::error!("Failed to connect: {}", e),
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, shutting down");
        }
    }
}
