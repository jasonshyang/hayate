use crate::bybit::types::{BybitMessage, BYBIT_ENDPOINT};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use transport::{WsClient, WsHandler};

pub struct BybitClient {
    inner: WsClient<BybitWsHandler>,
}

pub struct BybitWsHandler {
    /// Outbound sender
    msg_sender: mpsc::UnboundedSender<BybitMessage>,
    /// WebSocket sender
    ws_sender: Option<mpsc::UnboundedSender<Message>>,
}

// TODO: allow subscribing to multiple topics
impl BybitClient {
    pub fn new(update_sender: mpsc::UnboundedSender<BybitMessage>) -> Self {
        let handler = BybitWsHandler::new(update_sender);
        let client = WsClient::new(BYBIT_ENDPOINT, handler);
        Self { inner: client }
    }

    pub fn new_with_shutdown(
        update_sender: mpsc::UnboundedSender<BybitMessage>,
        shutdown: CancellationToken,
    ) -> Self {
        let handler = BybitWsHandler::new(update_sender);
        let client = WsClient::new_with_shutdown(BYBIT_ENDPOINT, handler, shutdown);
        Self { inner: client }
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        self.inner.connect().await
    }
}

#[async_trait::async_trait]
impl WsHandler for BybitWsHandler {
    async fn on_open(&mut self, sender: mpsc::UnboundedSender<Message>) -> anyhow::Result<()> {
        let depth = 50; // Depth of the order book
        let symbol = "BTCUSDT"; // Symbol to subscribe to
        let topic = format!("orderbook.{}.{}", depth, symbol);

        // TODO: fix hardcoded subscription message
        let subscribe_msg = serde_json::json!({
            "req_id": "test", // optional
            "op": "subscribe",
            "args": [topic]
        })
        .to_string();

        sender.send(Message::Text(subscribe_msg.into()))?;
        tracing::info!("Subscribed to orderbook updates for {}", symbol);

        self.ws_sender = Some(sender);
        Ok(())
    }

    async fn on_message(&mut self, message: Message) -> anyhow::Result<()> {
        if let Some(ws_sender) = &self.ws_sender {
            // Handle incoming WebSocket messages
            match message {
                Message::Text(text) => {
                    tracing::info!("Received text message: {}", text);
                    let msg: BybitMessage = serde_json::from_str(&text)
                        .map_err(|e| anyhow::anyhow!("Failed to parse message: {}", e))?;
                    tracing::info!("Parsed message: {:?}", msg);
                    self.msg_sender
                        .send(msg)
                        .map_err(|e| anyhow::anyhow!("Failed to send update: {}", e))?;
                }
                Message::Ping(ping) => {
                    tracing::info!("Received ping: {:?}", ping);
                    ws_sender.send(Message::Pong(ping))?;
                }
                Message::Close(_) => {
                    tracing::info!("WebSocket connection closed");
                    self.on_close().await?;
                }
                _ => {
                    tracing::warn!("Received unsupported message type: {:?}", message);
                    return Err(anyhow::anyhow!("Unsupported message type received"));
                }
            }
        } else {
            return Err(anyhow::anyhow!("WebSocket received message before open"));
        }

        Ok(())
    }

    async fn on_close(&mut self) -> anyhow::Result<()> {
        tracing::info!("Bybit Websocket connection closed");
        self.ws_sender = None;

        // TODO: Handle any cleanup if necessary
        Ok(())
    }
}

impl BybitWsHandler {
    pub fn new(update_sender: mpsc::UnboundedSender<BybitMessage>) -> Self {
        Self {
            msg_sender: update_sender,
            ws_sender: None,
        }
    }
}
