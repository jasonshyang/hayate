use std::collections::HashMap;

use hayate_core::traits::State;

use crate::models::{Decimal, Indicator, InternalEvent};

#[derive(Debug, Default)]
pub struct PriceState {
    price_indicators: HashMap<String, Box<dyn Indicator>>,
}

#[async_trait::async_trait]
impl State<InternalEvent> for PriceState {
    fn name(&self) -> &str {
        "price"
    }

    async fn sync(&mut self) -> anyhow::Result<()> {
        // TODO: Implement sync logic
        Ok(())
    }

    fn process_event(&mut self, event: InternalEvent) -> anyhow::Result<()> {
        match event {
            InternalEvent::TradeUpdate(trades) => {
                for trade in trades {
                    self.update(trade.price, trade.timestamp);
                }
            }
            InternalEvent::OrderBookUpdate(_)
            | InternalEvent::OrderPlaced(_)
            | InternalEvent::OrderFilled(_)
            | InternalEvent::OrderCancelled(_) => {}
        }

        Ok(())
    }
}

impl PriceState {
    pub fn new() -> Self {
        Self {
            price_indicators: HashMap::new(),
        }
    }

    pub fn add_indicator(&mut self, indicator: Box<dyn Indicator>) {
        self.price_indicators
            .insert(indicator.name().to_string(), indicator);
    }

    pub fn get_indicator(&self, name: &str) -> Option<&dyn Indicator> {
        self.price_indicators.get(name).map(|ind| ind.as_ref())
    }

    pub fn get_indicators(&self) -> &HashMap<String, Box<dyn Indicator>> {
        &self.price_indicators
    }

    pub fn update(&mut self, price: Decimal, timestamp: u64) {
        for indicator in self.price_indicators.values_mut() {
            indicator.update(price, timestamp);
        }
    }
}
