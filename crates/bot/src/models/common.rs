use crate::models::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Bid => write!(f, "bid"),
            Side::Ask => write!(f, "ask"),
        }
    }
}
