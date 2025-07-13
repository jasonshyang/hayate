use hayate_core::traits::Executor;
use tokio::sync::mpsc;

use crate::paper_trade::types::PaperExchangeMessage;

// TODO: add delay to simulate network latency
pub struct PaperExecutor {
    action_sender: mpsc::UnboundedSender<PaperExchangeMessage>,
}

#[async_trait::async_trait]
impl Executor<PaperExchangeMessage> for PaperExecutor {
    async fn execute(&self, action: PaperExchangeMessage) -> anyhow::Result<()> {
        if let Err(e) = self.action_sender.send(action) {
            tracing::info!("Paper exchange channel closed, stopping executor: {}", e);
        }

        Ok(())
    }
}
impl PaperExecutor {
    pub fn new(action_sender: mpsc::UnboundedSender<PaperExchangeMessage>) -> Self {
        Self { action_sender }
    }
}
