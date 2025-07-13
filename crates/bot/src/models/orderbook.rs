use std::collections::BTreeMap;

use crate::models::Decimal;

use super::Side;

#[derive(Debug)]
pub struct OrderBook {
    bids: BTreeMap<Decimal, Decimal>, // price -> total size
    asks: BTreeMap<Decimal, Decimal>, // price -> total size
    max_depth: usize,
}

impl OrderBook {
    pub fn new(max_depth: usize) -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            max_depth,
        }
    }

    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }

    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }

    pub fn best_price(&self, side: &Side) -> Option<Decimal> {
        match side {
            Side::Bid => self.best_bid(),
            Side::Ask => self.best_ask(),
        }
    }

    pub fn mid_price(&self) -> Option<Decimal> {
        let best_bid = self.best_bid()?;
        let best_ask = self.best_ask()?;
        Some((best_bid + best_ask) / 2.into())
    }

    pub fn bids(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.bids
    }

    pub fn asks(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.asks
    }

    pub fn bids_depth(&self) -> usize {
        self.bids.len()
    }

    pub fn asks_depth(&self) -> usize {
        self.asks.len()
    }

    pub fn insert(&mut self, side: Side, price: Decimal, size: Decimal) -> anyhow::Result<()> {
        if !size.is_positive() {
            return Err(anyhow::anyhow!("Size {} must be positive", size));
        }

        match side {
            Side::Bid => self.bids.insert(price, size),
            Side::Ask => self.asks.insert(price, size),
        };

        self.trim_levels();
        Ok(())
    }

    pub fn remove(&mut self, side: Side, price: Decimal) -> anyhow::Result<()> {
        let removed = match side {
            Side::Bid => self.bids.remove(&price),
            Side::Ask => self.asks.remove(&price),
        };

        if removed.is_none() {
            return Err(anyhow::anyhow!(
                "Level not found for side: {}, price: {}",
                side,
                price
            ));
        }

        Ok(())
    }

    pub fn adjust(&mut self, side: Side, price: Decimal, delta: Decimal) -> anyhow::Result<()> {
        let current_size = match side {
            Side::Bid => self.bids.get_mut(&price),
            Side::Ask => self.asks.get_mut(&price),
        };

        if let Some(size) = current_size {
            if *size + delta < Decimal::ZERO {
                return Err(anyhow::anyhow!(
                    "Cannot reduce size below zero, current size: {}, delta: {}",
                    size,
                    delta
                ));
            }
            *size += delta;

            if size.is_zero() {
                match side {
                    Side::Bid => self.bids.remove(&price),
                    Side::Ask => self.asks.remove(&price),
                };
            }
        } else {
            return Err(anyhow::anyhow!(
                "Level not found for side: {}, price: {}",
                side,
                price
            ));
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }

    pub fn trim_levels(&mut self) {
        while self.bids.len() > self.max_depth {
            let lowest_bid = *self.bids.keys().next().unwrap();
            self.bids.remove(&lowest_bid);
        }

        while self.asks.len() > self.max_depth {
            let highest_ask = *self.asks.keys().next_back().unwrap();
            self.asks.remove(&highest_ask);
        }
    }
}

#[cfg(test)]
mod orderbook_tests {
    use super::*;

    #[test]
    fn test_insert_and_remove() {
        let mut orderbook = OrderBook::new(5);

        orderbook.insert(Side::Bid, 100.into(), 1.into()).unwrap();
        orderbook.insert(Side::Bid, 99.into(), 2.into()).unwrap();
        orderbook.insert(Side::Bid, 98.into(), 3.into()).unwrap();

        orderbook.insert(Side::Ask, 101.into(), 1.into()).unwrap();
        orderbook.insert(Side::Ask, 102.into(), 2.into()).unwrap();
        orderbook.insert(Side::Ask, 103.into(), 3.into()).unwrap();

        assert_eq!(orderbook.best_bid(), Some(100.into()));
        assert_eq!(orderbook.best_ask(), Some(101.into()));

        orderbook.remove(Side::Bid, 100.into()).unwrap();
        assert_eq!(orderbook.best_bid(), Some(99.into()));

        orderbook.remove(Side::Ask, 101.into()).unwrap();
        orderbook.remove(Side::Ask, 102.into()).unwrap();
        assert_eq!(orderbook.best_ask(), Some(103.into()));

        orderbook.remove(Side::Ask, 103.into()).unwrap();
        assert!(orderbook.best_ask().is_none());
    }

    #[test]
    fn test_adjust() {
        let mut orderbook = OrderBook::new(5);

        orderbook.insert(Side::Bid, 100.into(), 1.into()).unwrap();
        orderbook.insert(Side::Bid, 99.into(), 2.into()).unwrap();

        orderbook.insert(Side::Ask, 101.into(), 3.into()).unwrap();
        orderbook.insert(Side::Ask, 102.into(), 2.into()).unwrap();

        orderbook
            .adjust(Side::Bid, 100.into(), Decimal::from_f64_unchecked(0.5))
            .unwrap();
        assert_eq!(
            orderbook.bids.get(&100.into()),
            Some(&Decimal::from_f64_unchecked(1.5))
        );

        orderbook
            .adjust(Side::Ask, 101.into(), Decimal::from_f64_unchecked(-0.5))
            .unwrap();
        assert_eq!(
            orderbook.asks.get(&101.into()),
            Some(&Decimal::from_f64_unchecked(2.5))
        );

        orderbook
            .adjust(Side::Bid, 99.into(), Decimal::from_f64_unchecked(-2.0))
            .unwrap();
        assert!(orderbook.bids.get(&99.into()).is_none());
    }

    #[test]
    fn test_trim() {
        let mut orderbook = OrderBook::new(2);

        orderbook.insert(Side::Bid, 100.into(), 1.into()).unwrap();
        orderbook.insert(Side::Bid, 98.into(), 3.into()).unwrap();
        orderbook.insert(Side::Bid, 99.into(), 2.into()).unwrap(); // This should trigger trim

        assert_eq!(orderbook.bids_depth(), 2);
        assert!(orderbook.bids.contains_key(&99.into()));
        assert!(orderbook.bids.contains_key(&100.into()));
        assert!(!orderbook.bids.contains_key(&98.into()));

        orderbook.insert(Side::Ask, 101.into(), 1.into()).unwrap();
        orderbook.insert(Side::Ask, 103.into(), 3.into()).unwrap();
        orderbook.insert(Side::Ask, 102.into(), 2.into()).unwrap(); // This should trigger trim

        assert_eq!(orderbook.asks_depth(), 2);
        assert!(orderbook.asks.contains_key(&101.into()));
        assert!(orderbook.asks.contains_key(&102.into()));
        assert!(!orderbook.asks.contains_key(&103.into()));
    }
}
