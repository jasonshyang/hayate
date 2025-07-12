use hayate_core::traits::Executor;
use tokio::sync::mpsc;

use crate::models::BotAction;

pub struct PaperExecutor {
    action_sender: mpsc::UnboundedSender<BotAction>,
}

#[async_trait::async_trait]
impl Executor<BotAction> for PaperExecutor {
    async fn execute(&self, action: BotAction) -> anyhow::Result<()> {
        self.action_sender
            .send(action)
            .map_err(|e| anyhow::anyhow!("Failed to send action: {}", e))?;

        Ok(())
    }
}
impl PaperExecutor {
    pub fn new(action_sender: mpsc::UnboundedSender<BotAction>) -> Self {
        Self { action_sender }
    }
}
