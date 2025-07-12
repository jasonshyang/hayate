use crate::models::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Side {
    #[default]
    Bid,
    Ask,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}

pub struct OrderEntry {
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
}

impl OrderEntry {
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

#[cfg(test)]
mod order_tests {
    use super::*;

    #[test]
    fn test_order_entry_creation() {
        let entry = OrderEntry::try_new(Side::Bid, 100, 1.0).unwrap();
        assert_eq!(entry.side, Side::Bid);
        assert_eq!(entry.price.to_string(), "100.000000");
        assert_eq!(entry.size.to_string(), "1.000000");
    }

    #[test]
    fn test_order_entry_from_decimal() {
        let price = Decimal::try_from(100).unwrap();
        let size = Decimal::try_from(1.0).unwrap();
        let entry = OrderEntry::new(Side::Ask, price, size);
        assert_eq!(entry.side, Side::Ask);
        assert_eq!(entry.price.to_string(), "100.000000");
        assert_eq!(entry.size.to_string(), "1.000000");
    }
}
