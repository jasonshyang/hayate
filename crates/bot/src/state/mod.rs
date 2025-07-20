mod orderbook;
mod pending_orders;
mod position;
mod price;

use hayate_core::traits::State;
pub use orderbook::*;
pub use pending_orders::*;
pub use position::*;
pub use price::*;

use crate::models::InternalEvent;

pub enum BotState {
    OrderBook(OrderBookState),
    Position(PositionState),
    PendingOrders(PendingOrdersState),
    Price(PriceState),
}

#[async_trait::async_trait]
impl State<InternalEvent> for BotState {
    fn name(&self) -> &str {
        match self {
            BotState::OrderBook(state) => state.name(),
            BotState::Position(state) => state.name(),
            BotState::PendingOrders(state) => state.name(),
            BotState::Price(state) => state.name(),
        }
    }

    async fn sync(&mut self) -> anyhow::Result<()> {
        match self {
            BotState::OrderBook(state) => state.sync().await,
            BotState::Position(state) => state.sync().await,
            BotState::PendingOrders(state) => state.sync().await,
            BotState::Price(state) => state.sync().await,
        }
    }

    fn process_event(&mut self, event: InternalEvent) -> anyhow::Result<()> {
        match self {
            BotState::OrderBook(state) => state.process_event(event),
            BotState::Position(state) => state.process_event(event),
            BotState::PendingOrders(state) => state.process_event(event),
            BotState::Price(state) => state.process_event(event),
        }
    }
}
