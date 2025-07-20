use std::str::FromStr;

use crate::models::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub oid: usize,
    pub symbol: String,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
    pub timestamp: u64,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}

impl FromStr for Side {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bid" => Ok(Side::Bid),
            "ask" => Ok(Side::Ask),
            "buy" => Ok(Side::Bid),
            "sell" => Ok(Side::Ask),
            _ => Err("Invalid side"),
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

impl Order {
    pub fn new(oid: usize, symbol: String, side: Side, price: Decimal, size: Decimal) -> Self {
        Self {
            oid,
            symbol,
            side,
            price,
            size,
        }
    }
}
