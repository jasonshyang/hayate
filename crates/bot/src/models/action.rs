use crate::models::{Decimal, Side};

#[derive(Debug, Clone)]
pub enum BotAction {
    PlaceOrder(PlaceOrder),
    CancelOrder(CancelOrder),
}

#[derive(Debug, Clone)]
pub struct PlaceOrder {
    pub symbol: String,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
}

#[derive(Debug, Clone)]
pub struct CancelOrder {
    pub symbol: String,
    pub oid: usize,
}
