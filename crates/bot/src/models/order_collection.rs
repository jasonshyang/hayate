use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::models::{Decimal, Order, Side};

/// Represents a collection of orders, allowing for efficient management
/// and retrieval of orders based on their price and side.
#[derive(Debug, Clone, Default)]
pub struct OrderCollection {
    bids: BTreeMap<Decimal, BTreeSet<usize>>,
    asks: BTreeMap<Decimal, BTreeSet<usize>>,
    registry: HashMap<usize, Order>,
}

impl OrderCollection {
    pub fn insert(&mut self, order: Order) {
        match order.side {
            Side::Bid => self.bids.entry(order.price).or_default().insert(order.oid),
            Side::Ask => self.asks.entry(order.price).or_default().insert(order.oid),
        };
        self.registry.insert(order.oid, order);
    }

    pub fn remove_by_oid(&mut self, oid: usize) -> Option<Order> {
        if let Some(order) = self.registry.remove(&oid) {
            match order.side {
                Side::Bid => {
                    if let Some(orders) = self.bids.get_mut(&order.price) {
                        orders.remove(&oid);
                        if orders.is_empty() {
                            self.bids.remove(&order.price);
                        }
                    }
                }
                Side::Ask => {
                    if let Some(orders) = self.asks.get_mut(&order.price) {
                        orders.remove(&oid);
                        if orders.is_empty() {
                            self.asks.remove(&order.price);
                        }
                    }
                }
            }
            Some(order)
        } else {
            None
        }
    }

    /// Reduces the size of an order by its OID. Returns `true` if the order was found and reduced,
    /// `false` if the order was not found or if the size was not positive.
    pub fn reduce_order_size(&mut self, oid: usize, size: Decimal) -> bool {
        if !size.is_positive() {
            return false;
        }

        if let Some(existing_order) = self.registry.get_mut(&oid) {
            if existing_order.size > size {
                existing_order.size -= size;
            } else {
                self.remove_by_oid(oid);
            }
            true
        } else {
            false
        }
    }

    pub fn get_order(&self, oid: usize) -> Option<&Order> {
        self.registry.get(&oid)
    }

    pub fn get_all_oids(&self) -> Vec<usize> {
        self.registry.keys().cloned().collect()
    }

    pub fn get_order_mut(&mut self, oid: usize) -> Option<&mut Order> {
        self.registry.get_mut(&oid)
    }

    pub fn get_best_ask_price(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }

    pub fn get_best_bid_price(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }

    pub fn get_best_ask_order(&self) -> Option<&Order> {
        let oid = self.get_best_ask_oid()?;
        self.registry.get(&oid)
    }

    pub fn get_best_bid_order(&self) -> Option<&Order> {
        let oid = self.get_best_bid_oid()?;
        self.registry.get(&oid)
    }

    pub fn get_best_ask_oid(&self) -> Option<usize> {
        if let Some((_, orders)) = self.asks.iter().next() {
            if let Some(&oid) = orders.iter().next() {
                return Some(oid);
            }
        }
        None
    }

    pub fn get_best_bid_oid(&self) -> Option<usize> {
        if let Some((_, orders)) = self.bids.iter().next_back() {
            if let Some(&oid) = orders.iter().next() {
                return Some(oid);
            }
        }
        None
    }

    pub fn pop_best_ask(&mut self) -> Option<Order> {
        let oid = self.get_best_ask_oid()?;
        self.remove_by_oid(oid)
    }

    pub fn pop_best_bid(&mut self) -> Option<Order> {
        let oid = self.get_best_bid_oid()?;
        self.remove_by_oid(oid)
    }

    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    pub fn len(&self) -> usize {
        self.registry.len()
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.registry.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Order> {
        self.registry.values()
    }

    pub fn bids_iter(&self) -> impl Iterator<Item = &Order> {
        self.bids.iter().flat_map(move |(_, orders)| {
            orders.iter().filter_map(move |oid| self.registry.get(oid))
        })
    }

    pub fn asks_iter(&self) -> impl Iterator<Item = &Order> {
        self.asks.iter().flat_map(move |(_, orders)| {
            orders.iter().filter_map(move |oid| self.registry.get(oid))
        })
    }

    pub fn for_each_bid_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Order),
    {
        for orders in self.bids.values_mut() {
            for oid in orders.iter() {
                if let Some(order) = self.registry.get_mut(oid) {
                    f(order);
                }
            }
        }
    }

    pub fn for_each_ask_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Order),
    {
        for orders in self.asks.values_mut() {
            for oid in orders.iter() {
                if let Some(order) = self.registry.get_mut(oid) {
                    f(order);
                }
            }
        }
    }
}
