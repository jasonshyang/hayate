use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

#[async_trait::async_trait]
pub trait WsHandler: Send + Sync {
    async fn on_open(&mut self, sender: mpsc::UnboundedSender<Message>) -> anyhow::Result<()>;
    async fn on_message(&mut self, message: Message) -> anyhow::Result<()>;
    async fn on_close(&mut self) -> anyhow::Result<()>;
}

pub struct WsClient<H> {
    url: String,
    handler: H,
    shutdown: CancellationToken,
}

impl<H> WsClient<H>
where
    H: WsHandler + 'static,
{
    pub fn new(url: impl Into<String>, handler: H) -> Self {
        let shutdown = CancellationToken::new();
        Self {
            url: url.into(),
            handler,
            shutdown,
        }
    }

    pub fn new_with_shutdown(
        url: impl Into<String>,
        handler: H,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            url: url.into(),
            handler,
            shutdown,
        }
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url).await?;
        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        // Spawn task to send outbound messages
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = write.send(message).await {
                    tracing::error!("Error sending message: {}", e);
                    break;
                }
            }
        });

        // Call on_open handler which should send the initial message
        self.handler.on_open(tx).await?;

        // Connection loop
        loop {
            tokio::select! {
                message = read.next() => {
                    match message {
                        Some(Ok(msg)) => {
                            if let Err(e) = self.handler.on_message(msg).await {
                                tracing::error!("Error handling message: {}", e);
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            tracing::info!("WebSocket client closed by server.");
                            break;
                        }
                    }
                }
                _ = self.shutdown.cancelled() => {
                    tracing::info!("WebSocket client shutdown initiated.");
                    break;
                }
            }
        }

        // Call on_close handler
        if let Err(e) = self.handler.on_close().await {
            tracing::error!("Error during close: {}", e);
        } else {
            tracing::info!("WebSocket client closed gracefully.");
        }

        Ok(())
    }
}
