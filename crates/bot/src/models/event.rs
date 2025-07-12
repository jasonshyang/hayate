use crate::models::OrderEntry;

pub enum OrderBookEvent {
    Snapshot(OrderBookUpdate),
    Delta(OrderBookUpdate),
}

pub struct OrderBookUpdate {
    pub symbol: String,
    pub updated_at: u64,
    pub bids: Vec<OrderEntry>,
    pub asks: Vec<OrderEntry>,
}
