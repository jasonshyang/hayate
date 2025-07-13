use crate::models::{Decimal, OrderData, Side};

#[derive(Debug, Copy, Clone, Default)]
pub struct Position {
    pub side: Side,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub opened_at: u64,
    pub updated_at: u64,
}

impl Position {
    pub fn new(order: OrderData, timestamp: u64) -> Self {
        Self {
            side: order.side,
            size: order.size,
            entry_price: order.price,
            opened_at: timestamp,
            updated_at: timestamp,
        }
    }

    pub fn update(&mut self, order: OrderData, timestamp: u64) {
        if !self.is_open() {
            *self = Position::new(order, timestamp);
            return;
        }

        // Same side: increase position
        if order.side == self.side {
            let new_size = self.size + order.size;
            self.entry_price = (self.entry_price * self.size + order.price * order.size) / new_size;
            self.size = new_size;
        } else {
            // Opposite side: reduce position or flip
            match self.size.cmp(&order.size) {
                std::cmp::Ordering::Greater => {
                    // Reduce position
                    self.size -= order.size;
                }
                std::cmp::Ordering::Equal => {
                    // Close position
                    self.size = Decimal::ZERO;
                    self.entry_price = Decimal::ZERO;
                }
                std::cmp::Ordering::Less => {
                    // Flip position
                    self.side = order.side;
                    self.entry_price = order.price;
                    self.size = order.size - self.size;
                }
            }
        }

        self.updated_at = timestamp;
    }

    pub fn is_open(&self) -> bool {
        self.size > Decimal::ZERO
    }

    pub fn current_value(&self, current_price: Decimal) -> Decimal {
        if self.is_open() {
            current_price * self.size
        } else {
            Decimal::ZERO
        }
    }

    pub fn unrealized_pnl(&mut self, current_price: Decimal) -> Decimal {
        if !self.is_open() {
            return Decimal::ZERO;
        }

        let pnl_per_unit = match self.side {
            Side::Bid => current_price - self.entry_price,
            Side::Ask => self.entry_price - current_price,
        };

        pnl_per_unit * self.size
    }
}

#[cfg(test)]
mod position_tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let order = OrderData::try_new(Side::Bid, 100, 1.5).unwrap();
        let position = Position::new(order, 1622547800);
        assert_eq!(position.side, Side::Bid);
        assert_eq!(position.entry_price.to_string(), "100.000000");
        assert_eq!(position.size.to_string(), "1.500000");
        assert_eq!(position.opened_at, 1622547800);
    }

    #[test]
    fn test_position_add() {
        let order = OrderData::try_new(Side::Bid, 100, 1.5).unwrap();
        let mut position = Position::new(order, 1622547800);

        let add_order = OrderData::try_new(Side::Bid, 105, 0.5).unwrap();
        position.update(add_order, 1622547801);

        assert_eq!(position.size.to_string(), "2.000000");
        // (100 * 1.5 + 105 * 0.5) / 2.0 = 101.25
        assert_eq!(position.entry_price.to_string(), "101.250000");
        assert_eq!(position.updated_at, 1622547801);
    }

    #[test]
    fn test_position_partial_reduce() {
        let order = OrderData::try_new(Side::Bid, 100, 2.0).unwrap();
        let mut position = Position::new(order, 1622547800);

        let reduce_order = OrderData::try_new(Side::Ask, 105, 1.0).unwrap();
        position.update(reduce_order, 1622547801);

        assert_eq!(position.size.to_string(), "1.000000");
        assert_eq!(position.entry_price.to_string(), "100.000000");
        assert_eq!(position.updated_at, 1622547801);
    }

    #[test]
    fn test_position_full_reduce() {
        let order = OrderData::try_new(Side::Bid, 100, 2.0).unwrap();
        let mut position = Position::new(order, 1622547800);

        let reduce_order = OrderData::try_new(Side::Ask, 105, 2.0).unwrap();
        position.update(reduce_order, 1622547801);

        assert_eq!(position.size.to_string(), "0.000000");
        assert_eq!(position.entry_price.to_string(), "0.000000");
        assert_eq!(position.updated_at, 1622547801);
        assert!(!position.is_open());
    }

    #[test]
    fn test_position_flip() {
        let order = OrderData::try_new(Side::Bid, 100, 2.0).unwrap();
        let mut position = Position::new(order, 1622547800);

        let flip_order = OrderData::try_new(Side::Ask, 110, 3.0).unwrap();
        position.update(flip_order, 1622547801);

        assert_eq!(position.side, Side::Ask);
        assert_eq!(position.size.to_string(), "1.000000");
        assert_eq!(position.entry_price.to_string(), "110.000000");
        assert_eq!(position.updated_at, 1622547801);
    }

    #[test]
    fn test_position_unrealized_pnl() {
        let order = OrderData::try_new(Side::Bid, 100, 2.0).unwrap();
        let mut position = Position::new(order, 1622547800);
        let pnl = position.unrealized_pnl(110.into());
        assert_eq!(pnl.to_string(), "20.000000"); // (110 - 100) * 2.0 = 20.0
        let pnl = position.unrealized_pnl(100.into());
        assert_eq!(pnl.to_string(), "0.000000"); // (100 - 100) * 2.0 = 0.0
    }
}
