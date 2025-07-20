use hayate_core::traits::{Bot, Input};

use crate::{
    models::{BotAction, CancelOrder, Decimal, PlaceOrder, Side},
    state::BotState,
};

/// Simple Market Making Bot
/// This bot places limit orders on both sides of the order book
/// at a specified spread from the mid price.
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

        for oid in input.pending_oids {
            actions.push(BotAction::CancelOrder(CancelOrder {
                symbol: self.symbol.clone(),
                oid,
            }));
        }

        let bid_spread = Decimal::from(self.bid_spread);
        let ask_spread = Decimal::from(self.ask_spread);

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

        actions.push(BotAction::PlaceOrder(PlaceOrder {
            symbol: self.symbol.clone(),
            price: bid_price,
            size: self.order_amount,
            side: Side::Bid,
        }));

        actions.push(BotAction::PlaceOrder(PlaceOrder {
            symbol: self.symbol.clone(),
            price: ask_price,
            size: self.order_amount,
            side: Side::Ask,
        }));

        Ok(actions)
    }
}

pub struct SMMInput {
    mid_price: Option<Decimal>,
    pending_oids: Vec<usize>,
}

impl Input<BotState> for SMMInput {
    fn empty() -> Self {
        SMMInput {
            mid_price: None,
            pending_oids: Vec::new(),
        }
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
            BotState::PendingOrders(pending_orders) => {
                self.pending_oids = pending_orders.get_inner().get_all_oids();
            }
            BotState::Price(_) => {}
        }
        Ok(())
    }
}
