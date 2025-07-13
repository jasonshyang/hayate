use hayate_core::traits::{Bot, Input};

use crate::{
    models::{BotAction, Decimal, Order, Side},
    state::BotState,
};

pub struct SMMInput {
    mid_price: Option<Decimal>,
}

impl Input<BotState> for SMMInput {
    fn empty() -> Self {
        SMMInput { mid_price: None }
    }

    fn read_state(&mut self, state: &BotState) -> anyhow::Result<()> {
        match state {
            BotState::OrderBook(order_book_state) => {
                if let Some(mid_price) = order_book_state.get_mid_price() {
                    self.mid_price = Some(mid_price);
                } else {
                    return Err(anyhow::anyhow!("Mid price not available in OrderBookState"));
                }
            }
            BotState::Position(position) => {
                // TODO: budget check
                tracing::debug!("Reading position state: {:?}", position.get_inner());
            }
        }
        Ok(())
    }
}

pub struct SMM {
    pub interval_ms: u64,
    pub symbol: String,
    pub order_amount: Decimal,
    pub bid_spread: f64,
    pub ask_spread: f64,
}

impl Bot<SMMInput, BotAction> for SMM {
    fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    fn evaluate(&self, input: SMMInput) -> anyhow::Result<Vec<BotAction>> {
        let mut actions = Vec::new();

        let bid_spread = Decimal::try_from(self.bid_spread).unwrap();
        let ask_spread = Decimal::try_from(self.ask_spread).unwrap();

        let mid_price = input
            .mid_price
            .ok_or_else(|| anyhow::anyhow!("Mid price not available"))?;
        let bid_price = mid_price - bid_spread;
        let ask_price = mid_price + ask_spread;

        tracing::debug!(
            "Current mid price: {}, placing orders at bid: {}, ask: {}",
            mid_price,
            bid_price,
            ask_price
        );

        actions.push(BotAction::PlaceOrder(Order {
            symbol: self.symbol.clone(),
            price: bid_price,
            size: self.order_amount,
            side: Side::Bid,
        }));

        actions.push(BotAction::PlaceOrder(Order {
            symbol: self.symbol.clone(),
            price: ask_price,
            size: self.order_amount,
            side: Side::Ask,
        }));

        // TODO: implement order cancellation

        Ok(actions)
    }
}
