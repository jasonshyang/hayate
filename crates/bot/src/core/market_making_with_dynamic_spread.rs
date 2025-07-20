use hayate_core::traits::{Bot, Input};

use crate::{
    models::{BotAction, CancelOrder, Decimal, Natr, PlaceOrder, Rsi, Side},
    state::BotState,
};

/// Dynamic Spread Market Making Bot
/// This bot places limit orders on both sides of the order book
/// at a dynamically spread.
///
/// The spread is adjusted for volatility factor using NATR and reference price is
/// skewed based on trend factor using RSI.
pub struct DynamicSpreadMM {
    pub interval_ms: u64,
    pub symbol: String,
    pub order_amount: Decimal,
    pub base_spread: Decimal,
    pub volatility_target: Decimal,
    pub skew_strength: Decimal,
}

impl Bot<DynamicSpreadMMInput, BotAction> for DynamicSpreadMM {
    fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    fn evaluate(&self, input: DynamicSpreadMMInput) -> anyhow::Result<Vec<BotAction>> {
        let mut actions = Vec::new();

        tracing::debug!(
            "Evaluating DynamicSpreadMM with mid_price: {:?}, rsi: {:?}, natr: {:?}",
            input.mid_price,
            input.rsi,
            input.natr
        );

        let mid_price = match input.mid_price {
            Some(price) => price,
            None => {
                tracing::info!("Mid price not available, skipping evaluation");
                return Ok(actions);
            }
        };

        let rsi = match input.rsi {
            Some(value) => value,
            None => {
                tracing::info!("RSI indicator not available, skipping evaluation");
                return Ok(actions);
            }
        };

        let natr = match input.natr {
            Some(value) => value,
            None => {
                tracing::info!("NATR indicator not available, skipping evaluation");
                return Ok(actions);
            }
        };

        for oid in input.pending_oids {
            actions.push(BotAction::CancelOrder(CancelOrder {
                symbol: self.symbol.clone(),
                oid,
            }));
        }

        let spread: Decimal = self.base_spread * (Decimal::ONE + natr / self.volatility_target);

        let skew: Decimal = match rsi {
            r if r < 30.0.into() => -self.skew_strength,
            r if r > 70.0.into() => self.skew_strength,
            _ => Decimal::ZERO,
        };

        let adjusted_mid_price = mid_price + (mid_price * skew);
        let bid_price = adjusted_mid_price - spread;
        let ask_price = adjusted_mid_price + spread;

        tracing::info!("DynamicSpreadMM Strategy placing order based on rsi: {}, natr: {}, mid price: {}, bid price: {}, ask price: {}", rsi, natr, mid_price, bid_price, ask_price);

        actions.push(BotAction::PlaceOrder(PlaceOrder {
            symbol: "BTCUSD".to_string(),
            price: bid_price,
            size: self.order_amount,
            side: Side::Bid,
        }));

        actions.push(BotAction::PlaceOrder(PlaceOrder {
            symbol: "BTCUSD".to_string(),
            price: ask_price,
            size: self.order_amount,
            side: Side::Ask,
        }));

        Ok(actions)
    }
}

#[derive(Debug)]
pub struct DynamicSpreadMMInput {
    mid_price: Option<Decimal>,
    rsi: Option<Decimal>,
    natr: Option<Decimal>,
    pending_oids: Vec<usize>,
}

impl Input<BotState> for DynamicSpreadMMInput {
    fn empty() -> Self {
        DynamicSpreadMMInput {
            mid_price: None,
            rsi: None,
            natr: None,
            pending_oids: Vec::new(),
        }
    }

    fn read_state(&mut self, state: &BotState) -> anyhow::Result<()> {
        match state {
            BotState::Price(price_state) => {
                if let Some(rsi) = price_state.get_indicator(Rsi::NAME) {
                    self.rsi = rsi.value();
                } else {
                    tracing::debug!("RSI indicator not found in price state");
                }

                if let Some(natr) = price_state.get_indicator(Natr::NAME) {
                    self.natr = natr.value();
                } else {
                    tracing::debug!("NATR indicator not found in price state");
                }
            }
            BotState::OrderBook(order_book_state) => {
                if let Some(mid_price) = order_book_state.get_mid_price() {
                    self.mid_price = Some(mid_price);
                } else {
                    tracing::debug!("Mid price not available in OrderBookState");
                }
            }
            BotState::PendingOrders(pending_orders) => {
                self.pending_oids = pending_orders.get_inner().get_all_oids();
            }
            BotState::Position(_) => {}
        }
        Ok(())
    }
}
