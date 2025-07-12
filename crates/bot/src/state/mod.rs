mod orderbook;
mod position;

use hayate_core::traits::State;
pub use orderbook::*;
pub use position::*;

use crate::models::BotEvent;

pub enum BotState {
    OrderBook(OrderBookState),
    Position(PositionState),
}

#[async_trait::async_trait]
impl State<BotEvent> for BotState {
    fn name(&self) -> &str {
        match self {
            BotState::OrderBook(state) => state.name(),
            BotState::Position(state) => state.name(),
        }
    }

    async fn sync(&mut self) -> anyhow::Result<()> {
        match self {
            BotState::OrderBook(state) => state.sync().await,
            BotState::Position(state) => state.sync().await,
        }
    }

    fn process_event(&mut self, event: BotEvent) -> anyhow::Result<()> {
        match self {
            BotState::OrderBook(state) => match event {
                BotEvent::OrderBookEvent(order_book_event) => state.process_event(order_book_event),
                _ => Err(anyhow::anyhow!("Invalid event type for OrderBookState")),
            },
            BotState::Position(state) => match event {
                BotEvent::BotTradeEvent(bot_trade_event) => state.process_event(bot_trade_event),
                _ => Err(anyhow::anyhow!("Invalid event type for PositionState")),
            },
        }
    }
}
