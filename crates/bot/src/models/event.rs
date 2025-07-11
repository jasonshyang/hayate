pub enum OrderBookEvent {
    Snapshot(OrderBookData),
    Update(OrderBookData),
}

pub struct OrderBookData {
    pub symbol: String,
    pub updated_at: u64,
    pub bids: Vec<(u64, f64)>,
    pub asks: Vec<(u64, f64)>,
}
