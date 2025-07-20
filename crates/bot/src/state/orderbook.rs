use hayate_core::traits::State;

use crate::models::{Decimal, InternalEvent, OrderBook, OrderBookEventKind, Side};

#[derive(Debug)]
pub struct OrderBookState {
    inner: OrderBook,
}

#[async_trait::async_trait]
impl State<InternalEvent> for OrderBookState {
    fn name(&self) -> &str {
        "orderbook"
    }

    async fn sync(&mut self) -> anyhow::Result<()> {
        // TODO: Implement sync logic
        Ok(())
    }

    fn process_event(&mut self, event: InternalEvent) -> anyhow::Result<()> {
        match event {
            InternalEvent::OrderBookUpdate(event) => match event.kind {
                OrderBookEventKind::Snapshot => {
                    self.update_snapshot(event.symbol, event.bids, event.asks)?;
                }
                OrderBookEventKind::Delta => {
                    self.update_delta(event.symbol, event.bids, event.asks)?;
                }
            },
            InternalEvent::OrderFilled(_)
            | InternalEvent::OrderPlaced(_)
            | InternalEvent::OrderCancelled(_)
            | InternalEvent::TradeUpdate(_) => {}
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

    pub fn get_inner(&self) -> &OrderBook {
        &self.inner
    }

    pub fn update_snapshot(
        &mut self,
        _symbol: String, // TODO: handle multiple symbols
        bids: Vec<(Decimal, Decimal)>,
        asks: Vec<(Decimal, Decimal)>,
    ) -> anyhow::Result<()> {
        self.inner.reset();

        for (price, size) in bids {
            self.inner.insert(Side::Bid, price, size)?;
        }

        for (price, size) in asks {
            self.inner.insert(Side::Ask, price, size)?;
        }

        Ok(())
    }

    pub fn update_delta(
        &mut self,
        _symbol: String, // TODO: handle multiple symbols
        bids: Vec<(Decimal, Decimal)>,
        asks: Vec<(Decimal, Decimal)>,
    ) -> anyhow::Result<()> {
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
        Ok(())
    }
}
