use hayate_core::traits::State;

use crate::models::{Decimal, InternalEvent, Position, Side};

#[derive(Debug, Default)]
pub struct PositionState {
    inner: Position,
}

#[async_trait::async_trait]
impl State<InternalEvent> for PositionState {
    fn name(&self) -> &str {
        "position"
    }

    async fn sync(&mut self) -> anyhow::Result<()> {
        // TODO: Implement sync logic
        Ok(())
    }

    fn process_event(&mut self, event: InternalEvent) -> anyhow::Result<()> {
        match event {
            InternalEvent::OrderFilled(fill) => {
                self.update_position(fill.side, fill.price, fill.size, fill.timestamp);
            }
            InternalEvent::OrderCancelled(_) => {}
            InternalEvent::OrderBookUpdate(_) => {}
            InternalEvent::OrderPlaced(_) => {}
        }

        Ok(())
    }
}

impl PositionState {
    pub fn new() -> Self {
        Self {
            inner: Position::default(),
        }
    }

    pub fn get_inner(&self) -> &Position {
        &self.inner
    }

    pub fn update_position(&mut self, side: Side, price: Decimal, size: Decimal, timestamp: u64) {
        if !self.inner.is_open() {
            self.inner = Position::new(side, price, size, timestamp);
        } else {
            self.inner.update(side, price, size, timestamp);
        }
    }
}
