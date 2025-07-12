use crate::models::{Decimal, OrderEntry, Side};

#[derive(Debug, Clone)]
pub enum BotEvent {
    OrderBookEvent(OrderBookEvent),
    BotTradeEvent(BotTradeEvent),
}

#[derive(Debug, Clone)]
pub enum OrderBookEvent {
    Snapshot(OrderBookUpdate),
    Delta(OrderBookUpdate),
}

#[derive(Debug, Clone)]
pub struct OrderBookUpdate {
    pub symbol: String,
    pub updated_at: u64,
    pub bids: Vec<OrderEntry>,
    pub asks: Vec<OrderEntry>,
}

#[derive(Debug, Clone)]
pub enum BotTradeEvent {
    TradeExecuted(TradeExecuted),
}

#[derive(Debug, Clone)]
pub struct TradeExecuted {
    pub symbol: String,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
    pub is_maker: bool,
    pub order_id: String,
    pub trade_id: String,
    pub timestamp: u64,
}

impl From<TradeExecuted> for OrderEntry {
    fn from(event: TradeExecuted) -> Self {
        OrderEntry::new(event.side, event.price, event.size)
    }
}
