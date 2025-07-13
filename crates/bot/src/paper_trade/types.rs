use crate::models::Order;

pub enum PaperExchangeMessage {
    PlaceOrder(Order),
    CancelOrder(Order),
    Close,
}
