mod natr;
mod rsi;

pub use natr::*;
pub use rsi::*;

use std::fmt::Debug;

use crate::models::Decimal;

pub trait Indicator: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn value(&self) -> Option<Decimal>;
    fn update(&mut self, price: Decimal, timestamp: u64);
    fn reset(&mut self);
}
