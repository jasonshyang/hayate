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
    pub bid_spread: Decimal,
    pub ask_spread: Decimal,
}

impl Bot<SMMInput, BotAction> for SMM {
    fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    fn evaluate(&self, input: SMMInput) -> anyhow::Result<Vec<BotAction>> {
        let mut actions = Vec::new();

        tracing::debug!(
            "Evaluating SMM with mid_price: {:?}, pending_oids: {:?}",
            input.mid_price,
            input.pending_oids
        );

        let mid_price = match input.mid_price {
            Some(price) => price,
            None => {
                tracing::info!("Mid price not available, skipping evaluation");
                return Ok(actions);
            }
        };

        for oid in input.pending_oids {
            actions.push(BotAction::CancelOrder(CancelOrder {
                symbol: self.symbol.clone(),
                oid,
            }));
        }

        let bid_price = mid_price - self.bid_spread;
        let ask_price = mid_price + self.ask_spread;

        tracing::info!(
            "SMM Strategy placing order based on mid price: {}, bid price: {}, ask price: {}",
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
                    tracing::debug!("Mid price not available in OrderBookState");
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
