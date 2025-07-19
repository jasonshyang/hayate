use crate::models::{Decimal, Order, Side};

#[derive(Debug, Clone)]
pub enum InternalEvent {
    OrderBookUpdate(OrderBookUpdate),
    OrderPlaced(Order),
    OrderFilled(Fill),
    OrderCancelled(Order),
}

#[derive(Debug, Clone)]
pub enum OrderBookEventKind {
    Snapshot,
    Delta,
}

#[derive(Debug, Clone)]
pub struct OrderBookUpdate {
    pub symbol: String,
    pub kind: OrderBookEventKind,
    pub updated_at: u64,
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Clone)]
pub struct Fill {
    pub oid: usize,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
    pub is_maker: bool,
    pub timestamp: u64,
}
