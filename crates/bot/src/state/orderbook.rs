use hayate_core::traits::State;

use crate::models::{Decimal, OrderBook, OrderBookEvent};

pub struct OrderBookState {
    inner: OrderBook,
}

#[async_trait::async_trait]
impl State<OrderBookEvent> for OrderBookState {
    fn name(&self) -> &str {
        "orderbook"
    }

    async fn sync(&mut self) -> anyhow::Result<()> {
        // TODO: Implement sync logic
        Ok(())
    }

    fn process_event(&mut self, event: OrderBookEvent) -> anyhow::Result<()> {
        match event {
            OrderBookEvent::Snapshot(update) => {
                let bids = update.bids;
                let asks = update.asks;
                self.inner.add_orders(bids);
                self.inner.add_orders(asks);
            }
            OrderBookEvent::Delta(update) => {
                let bids = update.bids;
                let asks = update.asks;
                self.inner.add_orders(bids);
                self.inner.add_orders(asks);
            }
        }

        Ok(())
    }
}

impl OrderBookState {
    pub fn new(max_depth: usize) -> Self {
        Self {
            inner: OrderBook::new(max_depth),
        }
    }

    pub fn get_mid_price(&self) -> Option<Decimal> {
        self.inner.mid_price()
    }
}
