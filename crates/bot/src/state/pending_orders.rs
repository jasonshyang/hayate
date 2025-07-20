use hayate_core::traits::State;

use crate::models::{Fill, InternalEvent, Order, OrderCollection};

#[derive(Debug, Default)]
pub struct PendingOrdersState {
    inner: OrderCollection,
}

#[async_trait::async_trait]
impl State<InternalEvent> for PendingOrdersState {
    fn name(&self) -> &str {
        "pending_orders"
    }

    async fn sync(&mut self) -> anyhow::Result<()> {
        // TODO: Implement sync logic
        Ok(())
    }

    fn process_event(&mut self, event: InternalEvent) -> anyhow::Result<()> {
        match event {
            InternalEvent::OrderPlaced(order) => {
                self.add_order(order);
            }
            InternalEvent::OrderFilled(fill) => {
                self.apply_fill(&fill)?;
            }
            InternalEvent::OrderCancelled(order) => {
                self.cancel_order(order.oid);
            }
            InternalEvent::OrderBookUpdate(_) | InternalEvent::TradeUpdate(_) => {}
        }

        Ok(())
    }
}

impl PendingOrdersState {
    pub fn new() -> Self {
        Self {
            inner: OrderCollection::default(),
        }
    }

    pub fn get_inner(&self) -> &OrderCollection {
        &self.inner
    }

    pub fn get_inner_mut(&mut self) -> &mut OrderCollection {
        &mut self.inner
    }

    pub fn add_order(&mut self, order: Order) {
        self.inner.insert(order);
    }

    pub fn apply_fill(&mut self, fill: &Fill) -> anyhow::Result<()> {
        if !self.inner.reduce_order_size(fill.oid, fill.size) {
            return Err(anyhow::anyhow!(
                "Failed to reduce order size for OID: {}",
                fill.oid
            ));
        }

        Ok(())
    }

    pub fn cancel_order(&mut self, oid: usize) -> Option<Order> {
        self.inner.remove_by_oid(oid)
    }
}
