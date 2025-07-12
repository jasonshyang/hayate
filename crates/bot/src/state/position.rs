use hayate_core::traits::State;

use crate::models::{BotTradeEvent, Position};

pub struct PositionState {
    inner: Position,
}

#[async_trait::async_trait]
impl State<BotTradeEvent> for PositionState {
    fn name(&self) -> &str {
        "position"
    }

    async fn sync(&mut self) -> anyhow::Result<()> {
        // TODO: Implement sync logic
        Ok(())
    }

    fn process_event(&mut self, event: BotTradeEvent) -> anyhow::Result<()> {
        match event {
            BotTradeEvent::TradeExecuted(execution) => {
                let timestamp = execution.timestamp;
                let order = execution.into();

                if !self.inner.is_open() {
                    self.inner = Position::new(order, timestamp);
                } else {
                    self.inner.update(order, timestamp);
                }
            }
        }

        Ok(())
    }
}
