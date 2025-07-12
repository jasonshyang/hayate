use std::collections::BTreeMap;

use crate::models::{Decimal, OrderEntry};

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

    pub fn add_order(&mut self, order: OrderEntry) {
        match order.side {
            Side::Bid => self.add_bid(order.price, order.size),
            Side::Ask => self.add_ask(order.price, order.size),
        }
    }

    pub fn add_orders(&mut self, orders: Vec<OrderEntry>) {
        for order in orders {
            self.add_order(order);
        }
    }

    pub fn remove_order(&mut self, order: OrderEntry) -> anyhow::Result<()> {
        match order.side {
            Side::Bid => self.remove_bid(order.price, order.size),
            Side::Ask => self.remove_ask(order.price, order.size),
        }
    }

    pub fn remove_orders(&mut self, orders: Vec<OrderEntry>) -> anyhow::Result<()> {
        for order in orders {
            self.remove_order(order)?;
        }
        Ok(())
    }

    pub fn add_bid(&mut self, price: Decimal, size: Decimal) {
        self.bids
            .entry(price)
            .and_modify(|existing_size| *existing_size += size)
            .or_insert(size);

        self.trim_bids();
    }

    pub fn add_ask(&mut self, price: Decimal, size: Decimal) {
        self.asks
            .entry(price)
            .and_modify(|existing_size| *existing_size += size)
            .or_insert(size);

        self.trim_asks();
    }

    pub fn remove_bid(&mut self, price: Decimal, size: Decimal) -> anyhow::Result<()> {
        match self.bids.get_mut(&price) {
            Some(existing_size) => {
                if *existing_size < size {
                    return Err(anyhow::anyhow!("Size to remove exceeds existing size"));
                }

                *existing_size -= size;

                if *existing_size == Decimal::ZERO {
                    self.bids.remove(&price);
                }

                Ok(())
            }
            None => Err(anyhow::anyhow!("Bid not found for price: {}", price)),
        }
    }

    pub fn remove_ask(&mut self, price: Decimal, size: Decimal) -> anyhow::Result<()> {
        match self.asks.get_mut(&price) {
            Some(existing_size) => {
                if *existing_size < size {
                    return Err(anyhow::anyhow!("Size to remove exceeds existing size"));
                }

                *existing_size -= size;

                if *existing_size == Decimal::ZERO {
                    self.asks.remove(&price);
                }

                Ok(())
            }
            None => Err(anyhow::anyhow!("Ask not found for price: {}", price)),
        }
    }

    pub fn add_bids(&mut self, bids: Vec<(Decimal, Decimal)>) {
        for (price, size) in bids {
            self.add_bid(price, size);
        }
    }

    pub fn add_asks(&mut self, asks: Vec<(Decimal, Decimal)>) {
        for (price, size) in asks {
            self.add_ask(price, size);
        }
    }

    pub fn remove_bids(&mut self, bids: Vec<(Decimal, Decimal)>) -> anyhow::Result<()> {
        for (price, size) in bids {
            self.remove_bid(price, size)?;
        }
        self.trim_bids();

        Ok(())
    }

    pub fn remove_asks(&mut self, asks: Vec<(Decimal, Decimal)>) -> anyhow::Result<()> {
        for (price, size) in asks {
            self.remove_ask(price, size)?;
        }
        self.trim_asks();

        Ok(())
    }

    fn trim_bids(&mut self) {
        while self.bids.len() > self.max_depth {
            let lowest_bid = *self.bids.keys().next().unwrap();
            self.bids.remove(&lowest_bid);
        }
    }

    fn trim_asks(&mut self) {
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
    fn test_orderbook() {
        let mut orderbook = OrderBook::new(5);

        orderbook.add_orders(vec![
            OrderEntry::try_new(Side::Bid, 100, 1.0).unwrap(),
            OrderEntry::try_new(Side::Bid, 99, 2.0).unwrap(),
            OrderEntry::try_new(Side::Bid, 98, 3.0).unwrap(),
        ]);
        orderbook.add_orders(vec![
            OrderEntry::try_new(Side::Ask, 101, 1.0).unwrap(),
            OrderEntry::try_new(Side::Ask, 102, 2.0).unwrap(),
            OrderEntry::try_new(Side::Ask, 109, 3.0).unwrap(),
        ]);
        assert_eq!(orderbook.best_bid(), Some(100.into()));
        assert_eq!(orderbook.best_ask(), Some(101.into()));

        // Expect to drop the lowest bids (93, 94, 95)
        orderbook.add_orders(vec![
            OrderEntry::try_new(Side::Bid, 97, 1.0).unwrap(),
            OrderEntry::try_new(Side::Bid, 96, 2.0).unwrap(),
            OrderEntry::try_new(Side::Bid, 95, 3.0).unwrap(),
            OrderEntry::try_new(Side::Bid, 94, 4.0).unwrap(),
            OrderEntry::try_new(Side::Bid, 93, 5.0).unwrap(),
        ]);
        assert_eq!(orderbook.best_bid(), Some(100.into()));
        assert_eq!(orderbook.bids_depth(), 5);
        assert_eq!(orderbook.bids.keys().next(), Some(&96.into()));

        // Expect to drop the highest asks (107, 108, 109)
        orderbook.add_orders(vec![
            OrderEntry::try_new(Side::Ask, 99, 1.0).unwrap(),
            OrderEntry::try_new(Side::Ask, 105, 2.0).unwrap(),
            OrderEntry::try_new(Side::Ask, 106, 3.0).unwrap(),
            OrderEntry::try_new(Side::Ask, 107, 4.0).unwrap(),
            OrderEntry::try_new(Side::Ask, 108, 5.0).unwrap(),
        ]);
        assert_eq!(orderbook.best_ask(), Some(99.into()));
        assert_eq!(orderbook.asks_depth(), 5);
        assert_eq!(orderbook.asks.keys().next_back(), Some(&106.into()));

        // Remove some bids and asks
        orderbook
            .remove_orders(vec![
                OrderEntry::try_new(Side::Bid, 100, 0.5).unwrap(),
                OrderEntry::try_new(Side::Bid, 99, 1.0).unwrap(),
            ])
            .unwrap();
        assert_eq!(orderbook.best_bid(), Some(100.into()));
        assert_eq!(orderbook.bids_depth(), 5);

        orderbook
            .remove_orders(vec![
                OrderEntry::try_new(Side::Bid, 100, 0.5).unwrap(),
                OrderEntry::try_new(Side::Bid, 99, 0.5).unwrap(),
            ])
            .unwrap();
        assert_eq!(orderbook.best_bid(), Some(99.into()));
        assert_eq!(orderbook.bids_depth(), 4);

        orderbook
            .remove_orders(vec![
                OrderEntry::try_new(Side::Ask, 101, 0.5).unwrap(),
                OrderEntry::try_new(Side::Ask, 102, 1.0).unwrap(),
            ])
            .unwrap();
        assert_eq!(orderbook.best_ask(), Some(99.into()));
        assert_eq!(orderbook.asks_depth(), 5);

        orderbook
            .remove_orders(vec![
                OrderEntry::try_new(Side::Ask, 99, 1.0).unwrap(),
                OrderEntry::try_new(Side::Ask, 105, 0.5).unwrap(),
            ])
            .unwrap();
        assert_eq!(orderbook.best_ask(), Some(101.into()));
        assert_eq!(orderbook.asks_depth(), 4);
    }
}
