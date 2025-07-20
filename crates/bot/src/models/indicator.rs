use std::{collections::VecDeque, fmt::Debug};

use crate::models::Decimal;

pub trait Indicator: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn value(&self) -> Option<Decimal>;
    fn should_update(&self, timestamp: u64) -> bool;
    fn update(&mut self, price: Decimal, timestamp: u64);
    fn reset(&mut self);
}

#[derive(Debug, Clone)]
pub struct RSI {
    period: usize,
    values: VecDeque<Decimal>,
    current_value: Option<Decimal>,
    last_updated_at: u64,
    update_interval: u64,
}

impl RSI {
    pub fn new(period: usize, update_interval: u64) -> Self {
        Self {
            period,
            values: VecDeque::new(),
            current_value: None,
            last_updated_at: 0,
            update_interval,
        }
    }
}

impl Indicator for RSI {
    fn name(&self) -> &str {
        "rsi"
    }

    fn value(&self) -> Option<Decimal> {
        self.current_value
    }

    fn should_update(&self, timestamp: u64) -> bool {
        timestamp - self.last_updated_at >= self.update_interval
    }

    fn update(&mut self, price: Decimal, timestamp: u64) {
        if !self.should_update(timestamp) {
            return;
        }

        self.last_updated_at = timestamp;

        if self.values.len() == self.period {
            self.values.pop_front();
        }
        self.values.push_back(price);

        if self.values.len() < self.period {
            self.current_value = None;
            return;
        }

        let gains: Decimal = self
            .values
            .iter()
            .zip(self.values.iter().skip(1))
            .map(|(prev, curr)| {
                if curr > prev {
                    *curr - *prev
                } else {
                    Decimal::ZERO
                }
            })
            .sum();

        let losses: Decimal = self
            .values
            .iter()
            .zip(self.values.iter().skip(1))
            .map(|(prev, curr)| {
                if curr < prev {
                    *prev - *curr
                } else {
                    Decimal::ZERO
                }
            })
            .sum();

        let rs: Decimal = if losses.is_zero() {
            self.current_value = Some(Decimal::from(100.0));
            return;
        } else {
            gains / losses
        };

        self.current_value =
            Some(Decimal::from(100.0) - (Decimal::from(100.0) / (Decimal::ONE + rs)));
    }

    fn reset(&mut self) {
        self.values.clear();
        self.current_value = None;
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_rsi() {
        let prices = vec![
            Decimal::from(44.0),
            Decimal::from(44.15), // + 0.15, out of range
            Decimal::from(43.9),  // - 0.25 , out of range
            Decimal::from(44.05), // + 0.15, out of range
            Decimal::from(44.3),  // + 0.25
            Decimal::from(44.6),  // + 0.3
            Decimal::from(44.9),  // + 0.3
            Decimal::from(45.1),  // + 0.2
            Decimal::from(45.0),  // - 0.1
            Decimal::from(45.2),  // + 0.2
            Decimal::from(45.4),  // + 0.2
            Decimal::from(45.3),  // - 0.1
            Decimal::from(45.5),  // + 0.2
            Decimal::from(45.6),  // + 0.1
            Decimal::from(45.3),  // - 0.3
            Decimal::from(45.1),  // - 0.2
            Decimal::from(45.0),  // - 0.1
        ];

        let mut rsi = RSI::new(14, 100);
        let mut ts = chrono::Utc::now().timestamp() as u64;
        for price in prices {
            rsi.update(price, ts);
            ts += 100;
        }

        // Total gains: 0.25 + 0.3 + 0.3 + 0.2 + 0.2 + 0.2 + 0.2 + 0.1 = 1.75
        // Total losses: 0.1 + 0.1 + 0.3 + 0.2 + 0.1 = 0.8
        // RS = 1.75 / 0.8 = 2.1875
        // RSI = 100 - (100 / (1 + 2.1875))
        // RSI = 100 - (100 / 3.1875) = 68.627451
        assert_eq!(rsi.value(), Some(Decimal::from(68.627451)));
    }
}
