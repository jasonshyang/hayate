use std::collections::BTreeMap;

use super::Side;

#[derive(Debug)]
pub struct OrderBook {
    bids: BTreeMap<u64, f64>,
    asks: BTreeMap<u64, f64>,
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

    pub fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next_back().cloned()
    }

    pub fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().cloned()
    }

    pub fn best_price(&self, side: &Side) -> Option<u64> {
        match side {
            Side::Bid => self.best_bid(),
            Side::Ask => self.best_ask(),
        }
    }

    pub fn mid_price(&self) -> Option<u64> {
        let best_bid = self.best_bid()?;
        let best_ask = self.best_ask()?;
        Some((best_bid + best_ask) / 2)
    }

    pub fn bids(&self) -> &BTreeMap<u64, f64> {
        &self.bids
    }

    pub fn asks(&self) -> &BTreeMap<u64, f64> {
        &self.asks
    }

    pub fn bids_depth(&self) -> usize {
        self.bids.len()
    }

    pub fn asks_depth(&self) -> usize {
        self.asks.len()
    }

    pub fn add_order(&mut self, side: &Side, order: (u64, f64)) {
        match side {
            Side::Bid => self.add_bid(order.0, order.1),
            Side::Ask => self.add_ask(order.0, order.1),
        }
    }

    pub fn remove_order(&mut self, side: &Side, order: (u64, f64)) -> anyhow::Result<()> {
        match side {
            Side::Bid => self.remove_bid(order.0, order.1),
            Side::Ask => self.remove_ask(order.0, order.1),
        }
    }

    pub fn add_bid(&mut self, price: u64, size: f64) {
        self.bids
            .entry(price)
            .and_modify(|existing_size| *existing_size += size)
            .or_insert(size);

        self.trim_bids();
    }

    pub fn add_ask(&mut self, price: u64, size: f64) {
        self.asks
            .entry(price)
            .and_modify(|existing_size| *existing_size += size)
            .or_insert(size);

        self.trim_asks();
    }

    pub fn remove_bid(&mut self, price: u64, size: f64) -> anyhow::Result<()> {
        match self.bids.get_mut(&price) {
            Some(existing_size) => {
                if *existing_size < size {
                    return Err(anyhow::anyhow!("Size to remove exceeds existing size"));
                }

                *existing_size -= size;

                if *existing_size == 0.0 {
                    self.bids.remove(&price);
                }

                Ok(())
            }
            None => Err(anyhow::anyhow!("Bid not found for price: {}", price)),
        }
    }

    pub fn remove_ask(&mut self, price: u64, size: f64) -> anyhow::Result<()> {
        match self.asks.get_mut(&price) {
            Some(existing_size) => {
                if *existing_size < size {
                    return Err(anyhow::anyhow!("Size to remove exceeds existing size"));
                }

                *existing_size -= size;

                if *existing_size == 0.0 {
                    self.asks.remove(&price);
                }

                Ok(())
            }
            None => Err(anyhow::anyhow!("Ask not found for price: {}", price)),
        }
    }

    pub fn add_bids(&mut self, bids: Vec<(u64, f64)>) {
        for (price, size) in bids {
            self.add_bid(price, size);
        }
    }

    pub fn add_asks(&mut self, asks: Vec<(u64, f64)>) {
        for (price, size) in asks {
            self.add_ask(price, size);
        }
    }

    pub fn remove_bids(&mut self, bids: Vec<(u64, f64)>) -> anyhow::Result<()> {
        for (price, size) in bids {
            self.remove_bid(price, size)?;
        }
        self.trim_bids();

        Ok(())
    }

    pub fn remove_asks(&mut self, asks: Vec<(u64, f64)>) -> anyhow::Result<()> {
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
mod tests {
    use super::*;

    #[test]
    fn test_orderbook() {
        let mut orderbook = OrderBook::new(5);

        orderbook.add_bids(vec![(100, 1.0), (99, 2.0), (98, 3.0)]);
        orderbook.add_asks(vec![(101, 1.0), (102, 2.0), (109, 3.0)]);
        assert_eq!(orderbook.best_bid(), Some(100));
        assert_eq!(orderbook.best_ask(), Some(101));

        // Expect to drop the lowest bids (93, 94, 95)
        orderbook.add_bids(vec![(97, 1.0), (96, 2.0), (95, 3.0), (94, 4.0), (93, 5.0)]);
        assert_eq!(orderbook.best_bid(), Some(100));
        assert_eq!(orderbook.bids_depth(), 5);
        assert_eq!(orderbook.bids.keys().next(), Some(&96));

        // Expect to drop the highest asks (107, 108, 109)
        orderbook.add_asks(vec![
            (99, 1.0),
            (105, 2.0),
            (106, 3.0),
            (107, 4.0),
            (108, 5.0),
        ]);
        assert_eq!(orderbook.best_ask(), Some(99));
        assert_eq!(orderbook.asks_depth(), 5);
        assert_eq!(orderbook.asks.keys().next_back(), Some(&106));

        // Remove some bids and asks
        let _ = orderbook.remove_bids(vec![(100, 0.5), (99, 1.0)]);
        assert_eq!(orderbook.best_bid(), Some(100));
        assert_eq!(orderbook.bids_depth(), 5);
        let _ = orderbook.remove_bids(vec![(100, 0.5), (99, 0.5)]);
        assert_eq!(orderbook.best_bid(), Some(99));
        assert_eq!(orderbook.bids_depth(), 4);

        let _ = orderbook.remove_asks(vec![(101, 0.5), (102, 1.0)]);
        assert_eq!(orderbook.best_ask(), Some(99));
        assert_eq!(orderbook.asks_depth(), 5);
        let _ = orderbook.remove_asks(vec![(99, 1.0), (105, 0.5)]);
        assert_eq!(orderbook.best_ask(), Some(101));
        assert_eq!(orderbook.asks_depth(), 4);
    }
}
