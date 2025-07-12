use std::sync::Arc;

use bot::{
    collector::bybit::BybitCollector,
    core::simple_market_making::SMM,
    executor::paper_executor::PaperExecutor,
    models::{BotAction, Decimal},
    state::{BotState, OrderBookState},
};
use hayate_core::run::run_bot;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let market_making_bot = SMM {
        interval_ms: 1000,
        symbol: "BTCUSD".to_string(),
        order_amount: Decimal::from(1),
        bid_spread: 0.01,
        ask_spread: 0.01,
    };

    let bybit_collector = BybitCollector;

    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel();
    let paper_executor = PaperExecutor::new(action_tx);
    let orderbook_state = Arc::new(RwLock::new(BotState::OrderBook(OrderBookState::new(1042))));

    let mut set = run_bot(
        market_making_bot,
        vec![orderbook_state],
        vec![Box::new(bybit_collector)],
        vec![Box::new(paper_executor)],
    );

    tokio::spawn(async move {
        while let Some(action) = action_rx.recv().await {
            match action {
                BotAction::PlaceOrder(place_order) => {
                    println!("Placing order: {:?}", place_order);
                }
                BotAction::CancelOrder(cancel_order) => {
                    println!("Cancelling order: {:?}", cancel_order);
                }
            }
        }
    });

    while let Some(result) = set.join_next().await {
        match result {
            Ok(_) => println!("Bot task completed successfully."),
            Err(e) => eprintln!("Bot task encountered an error: {}", e),
        }
    }
}
