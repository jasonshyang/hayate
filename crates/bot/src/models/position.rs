use crate::models::{Decimal, Side};

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub side: Side,
    pub entry_price: Decimal,
    pub size: Decimal,
    pub opened_at: u64,
    pub updated_at: u64,
}

impl Position {
    pub fn new(side: Side, price: Decimal, size: Decimal, timestamp: u64) -> Self {
        Self {
            side,
            entry_price: price,
            size,
            opened_at: timestamp,
            updated_at: timestamp,
        }
    }

    pub fn is_open(&self) -> bool {
        self.size > Decimal::ZERO
    }

    pub fn update(&mut self, side: Side, price: Decimal, size: Decimal, timestamp: u64) {
        if !self.is_open() {
            *self = Position::new(side, price, size, timestamp);
            return;
        }

        // Same side: increase position
        if side == self.side {
            let new_size = self.size + size;
            self.entry_price = (self.entry_price * self.size + price * size) / new_size;
            self.size = new_size;
        } else {
            // Opposite side: reduce position or flip
            match self.size.cmp(&size) {
                std::cmp::Ordering::Greater => {
                    // Reduce position
                    self.size -= size;
                }
                std::cmp::Ordering::Equal => {
                    // Close position
                    self.size = Decimal::ZERO;
                    self.entry_price = Decimal::ZERO;
                }
                std::cmp::Ordering::Less => {
                    // Flip position
                    self.side = side;
                    self.entry_price = price;
                    self.size = size - self.size;
                }
            }
        }

        self.updated_at = timestamp;
    }

    pub fn current_value(&self, current_price: Decimal) -> Decimal {
        if self.is_open() {
            current_price * self.size
        } else {
            Decimal::ZERO
        }
    }

    pub fn unrealized_pnl(&self, current_price: Decimal) -> Decimal {
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

impl Default for Position {
    fn default() -> Self {
        Self {
            side: Side::Bid,
            size: Decimal::ZERO,
            entry_price: Decimal::ZERO,
            opened_at: 0,
            updated_at: 0,
        }
    }
}

#[cfg(test)]
mod position_tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let position = Position::new(Side::Bid, 100.into(), 1.5.try_into().unwrap(), 1622547800);
        assert_eq!(position.side, Side::Bid);
        assert_eq!(position.entry_price.to_string(), "100.000000");
        assert_eq!(position.size.to_string(), "1.500000");
        assert_eq!(position.opened_at, 1622547800);
    }

    #[test]
    fn test_position_add() {
        let mut position =
            Position::new(Side::Bid, 100.into(), 1.5.try_into().unwrap(), 1622547800);

        position.update(Side::Bid, 105.into(), 0.5.try_into().unwrap(), 1622547801);

        assert_eq!(position.size.to_string(), "2.000000");
        // (100 * 1.5 + 105 * 0.5) / 2.0 = 101.25
        assert_eq!(position.entry_price.to_string(), "101.250000");
        assert_eq!(position.updated_at, 1622547801);
    }

    #[test]
    fn test_position_partial_reduce() {
        let mut position =
            Position::new(Side::Bid, 100.into(), 2.0.try_into().unwrap(), 1622547800);

        position.update(Side::Ask, 105.into(), 1.0.try_into().unwrap(), 1622547801);

        assert_eq!(position.size.to_string(), "1.000000");
        assert_eq!(position.entry_price.to_string(), "100.000000");
        assert_eq!(position.updated_at, 1622547801);
    }

    #[test]
    fn test_position_full_reduce() {
        let mut position =
            Position::new(Side::Bid, 100.into(), 2.0.try_into().unwrap(), 1622547800);

        position.update(Side::Ask, 105.into(), 2.0.try_into().unwrap(), 1622547801);

        assert_eq!(position.size.to_string(), "0.000000");
        assert_eq!(position.entry_price.to_string(), "0.000000");
        assert_eq!(position.updated_at, 1622547801);
        assert!(!position.is_open());
    }

    #[test]
    fn test_position_flip() {
        let mut position =
            Position::new(Side::Bid, 100.into(), 2.0.try_into().unwrap(), 1622547800);

        position.update(Side::Ask, 110.into(), 3.0.try_into().unwrap(), 1622547801);

        assert_eq!(position.side, Side::Ask);
        assert_eq!(position.size.to_string(), "1.000000");
        assert_eq!(position.entry_price.to_string(), "110.000000");
        assert_eq!(position.updated_at, 1622547801);
    }

    #[test]
    fn test_position_unrealized_pnl() {
        let position = Position::new(Side::Bid, 100.into(), 2.0.try_into().unwrap(), 1622547800);
        let pnl = position.unrealized_pnl(110.into());
        assert_eq!(pnl.to_string(), "20.000000"); // (110 - 100) * 2.0 = 20.0
        let pnl = position.unrealized_pnl(100.into());
        assert_eq!(pnl.to_string(), "0.000000"); // (100 - 100) * 2.0 = 0.0
    }
}
