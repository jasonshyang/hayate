use hayate_core::traits::{Bot, Input};

use crate::{
    models::{BotAction, Decimal},
    state::BotState,
};

pub struct DynamicSpreadMM {
    pub interval_ms: u64,
}

impl Bot<DynamicSpreadMMInput, BotAction> for DynamicSpreadMM {
    fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    fn evaluate(&self, input: DynamicSpreadMMInput) -> anyhow::Result<Vec<BotAction>> {
        let actions = Vec::new();

        tracing::info!("input: {:?}", input);

        Ok(actions)
    }
}

#[derive(Debug)]
pub struct DynamicSpreadMMInput {
    rsi: Option<Decimal>,
}

impl Input<BotState> for DynamicSpreadMMInput {
    fn empty() -> Self {
        DynamicSpreadMMInput { rsi: None }
    }

    fn read_state(&mut self, state: &BotState) -> anyhow::Result<()> {
        match state {
            BotState::Price(price_state) => {
                if let Some(rsi) = price_state.get_indicator("rsi") {
                    self.rsi = rsi.value();
                }
            }
            BotState::OrderBook(_) | BotState::Position(_) | BotState::PendingOrders(_) => {}
        }
        Ok(())
    }
}
