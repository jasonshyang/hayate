use hayate_core::traits::State;

use crate::models::{OrderBook, OrderBookEvent};

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
                self.inner.add_bids(bids);
                self.inner.add_asks(asks);

                Ok(())
            }
            OrderBookEvent::Update(update) => {
                let bids = update.bids;
                let asks = update.asks;
                self.inner.add_bids(bids);
                self.inner.add_asks(asks);

                Ok(())
            }
        }
    }
}
