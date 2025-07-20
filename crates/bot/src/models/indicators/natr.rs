use std::collections::VecDeque;

use crate::models::{Decimal, Indicator};

#[derive(Debug, Clone)]
pub struct Natr {
    period: usize,
    true_ranges: VecDeque<Decimal>,
    current_value: Option<Decimal>,
    last_closed_at: u64,
    update_interval: u64,
    // TODO: move this out to a Candle struct
    high: Decimal,
    low: Decimal,
    open: Decimal,
    close: Decimal,
}

impl Natr {
    pub const NAME: &'static str = "natr";

    pub fn new(period: usize, update_interval: u64) -> Self {
        Self {
            period,
            true_ranges: VecDeque::new(),
            current_value: None,
            last_closed_at: 0,
            update_interval,
            high: Decimal::ZERO,
            low: Decimal::ZERO,
            open: Decimal::ZERO,
            close: Decimal::ZERO,
        }
    }

    fn should_make_new_candle(&self, timestamp: u64) -> bool {
        timestamp - self.last_closed_at >= self.update_interval
    }

    fn update_true_range(&mut self) {
        let hl = self.high - self.low;
        let hc = (self.high - self.open).abs();
        let lc = (self.low - self.open).abs();
        let tr = hl.max(hc).max(lc);

        if self.true_ranges.len() == self.period {
            self.true_ranges.pop_front();
        }
        self.true_ranges.push_back(tr);
    }
}

impl Indicator for Natr {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn value(&self) -> Option<Decimal> {
        self.current_value
    }

    fn update(&mut self, price: Decimal, timestamp: u64) {
        if self.last_closed_at == 0 {
            self.open = price;
            self.high = price;
            self.low = price;
            self.close = price;
            self.last_closed_at = timestamp;

            return;
        } else {
            self.high = self.high.max(price);
            self.low = self.low.min(price);
            self.close = price;
        }

        if self.should_make_new_candle(timestamp) {
            self.update_true_range();

            if self.true_ranges.len() < self.period {
                self.current_value = None;
            } else {
                let tr_sum: Decimal = self.true_ranges.iter().cloned().sum();

                let atr = tr_sum / Decimal::from(self.period as u64);
                self.current_value = Some((atr / self.close) * Decimal::from(100.0));
            }

            // Reset for the next candle
            self.open = self.close;
            self.high = price;
            self.low = price;
            self.last_closed_at = timestamp;
        }
    }

    fn reset(&mut self) {
        self.true_ranges.clear();
        self.current_value = None;
        self.last_closed_at = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natr() {
        let mut natr = Natr::new(4, 1000);
        let prices = vec![
            Decimal::from(100.0), // Initial price
            // Bar 1: Open 100, High 102, Low 98, Close 100
            Decimal::from(100.0),
            Decimal::from(102.0),
            Decimal::from(98.0),
            Decimal::from(100.0),
            // Bar 2: Open 100, High 105, Low 99, Close 102
            Decimal::from(102.0),
            Decimal::from(105.0),
            Decimal::from(99.0),
            Decimal::from(102.0),
            // Bar 3: Open 102, High 107, Low 100, Close 105
            Decimal::from(105.0),
            Decimal::from(107.0),
            Decimal::from(100.0),
            Decimal::from(105.0),
            // Bar 4: Open 105, High 106, Low 101, Close 103
            Decimal::from(103.0),
            Decimal::from(106.0),
            Decimal::from(101.0),
            Decimal::from(103.0),
        ];

        for (i, price) in prices.iter().enumerate() {
            natr.update(*price, 1 + (i as u64) * 250);
        }

        // Bar 1 TR = 4, Bar 2 TR = 6, Bar 3 TR = 7, Bar 4 TR = 5
        // ATR = (4 + 6 + 7 + 5) / 4 = 5.5
        // NATR = (5.5 / 103) * 100 = 5.339800
        assert_eq!(natr.value(), Some(Decimal::from(5.339800)));
    }
}
