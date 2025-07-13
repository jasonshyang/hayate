use crate::models::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Side {
    #[default]
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct OrderData {
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub data: OrderData,
    pub timestamp: u64,
}

impl OrderData {
    pub fn new(side: Side, price: Decimal, size: Decimal) -> Self {
        Self { side, price, size }
    }

    pub fn try_new(
        side: Side,
        price: impl TryInto<Decimal>,
        size: impl TryInto<Decimal>,
    ) -> anyhow::Result<Self> {
        let price = price
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid price conversion"))?;
        let size = size
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid size conversion"))?;

        Ok(Self { side, price, size })
    }
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

#[cfg(test)]
mod order_tests {
    use super::*;

    #[test]
    fn test_order_data_creation() {
        let order = OrderData::try_new(Side::Bid, 100, 1.0).unwrap();
        assert_eq!(order.side, Side::Bid);
        assert_eq!(order.price.to_string(), "100.000000");
        assert_eq!(order.size.to_string(), "1.000000");
    }

    #[test]
    fn test_order_data_from_decimal() {
        let price = Decimal::try_from(100).unwrap();
        let size = Decimal::try_from(1.0).unwrap();
        let order = OrderData::new(Side::Ask, price, size);
        assert_eq!(order.side, Side::Ask);
        assert_eq!(order.price.to_string(), "100.000000");
        assert_eq!(order.size.to_string(), "1.000000");
    }
}
