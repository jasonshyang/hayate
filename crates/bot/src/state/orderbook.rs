use hayate_core::traits::State;

use crate::models::{Decimal, OrderBook, OrderBookEvent, Side};

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
                self.inner.reset();

                let bids = update.bids;
                let asks = update.asks;

                for (price, size) in bids {
                    self.inner.insert(Side::Bid, price, size)?;
                }

                for (price, size) in asks {
                    self.inner.insert(Side::Bid, price, size)?;
                }
            }
            OrderBookEvent::Delta(update) => {
                let bids = update.bids;
                let asks = update.asks;

                for (price, size) in bids {
                    if size.is_zero() {
                        self.inner.remove(Side::Bid, price)?;
                    } else {
                        self.inner.insert(Side::Bid, price, size)?;
                    }
                }

                for (price, size) in asks {
                    if size.is_zero() {
                        self.inner.remove(Side::Ask, price)?;
                    } else {
                        self.inner.insert(Side::Ask, price, size)?;
                    }
                }
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
