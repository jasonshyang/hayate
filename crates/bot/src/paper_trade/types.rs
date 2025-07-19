use crate::models::{CancelOrder, PlaceOrder};

pub enum PaperExchangeMessage {
    PlaceOrder(PlaceOrder),
    CancelOrder(CancelOrder),
    Close,
}
