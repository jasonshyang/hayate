use std::sync::Arc;

use bot::{
    collector::{bybit_collector::BybitCollector, paper_collector::PaperCollector},
    core::simple_market_making::SMM,
    executor::paper_executor::PaperExecutor,
    models::{BotAction, Decimal},
    paper_trade::{paper_exchange::PaperExchange, types::PaperExchangeMessage},
    state::{BotState, OrderBookState, PendingOrdersState, PositionState},
};
use hayate_core::{mappers::ExecutorMap, run::run_bot};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let market_making_bot = SMM {
        interval_ms: 1000,
        symbol: "BTCUSD".to_string(),
        order_amount: Decimal::from(10),
        bid_spread: 0.01,
        ask_spread: 0.01,
    };

    // Create a channel for sending messages to the PaperExchange
    let (msg_tx, msg_rx) = tokio::sync::mpsc::unbounded_channel();

    // Shutdown
    let shutdown = CancellationToken::new();

    let bybit_collector = BybitCollector::new(shutdown.clone());
    let mut paper_exchange = PaperExchange::new();
    let paper_collector = PaperCollector::new(paper_exchange.subscribe());
    let paper_executor = ExecutorMap::new(
        Box::new(PaperExecutor::new(msg_tx)),
        |action: BotAction| match action {
            BotAction::PlaceOrder(order) => Some(PaperExchangeMessage::PlaceOrder(order)),
            BotAction::CancelOrder(order) => Some(PaperExchangeMessage::CancelOrder(order)),
        },
    );
    let orderbook_state = Arc::new(RwLock::new(BotState::OrderBook(OrderBookState::new(1024))));
    let position_state = Arc::new(RwLock::new(BotState::Position(PositionState::new())));
    let pending_orders_state = Arc::new(RwLock::new(BotState::PendingOrders(
        PendingOrdersState::new(),
    )));

    let mut set = run_bot(
        market_making_bot,
        vec![orderbook_state, position_state, pending_orders_state],
        vec![Box::new(paper_collector)],
        vec![Box::new(paper_executor)],
        shutdown.clone(),
    );

    let shutdown_signal = shutdown.clone();
    set.spawn(async move {
        tracing::info!("Starting PaperExchange...");
        if let Err(e) = paper_exchange
            .run_with_shutdown(bybit_collector, msg_rx, shutdown_signal)
            .await
        {
            tracing::error!("PaperExchange encountered an error: {}", e);
        }
        tracing::info!("PaperExchange stopped.");
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");
    tracing::info!("Shutdown signal received, stopping bot...");
    shutdown.cancel();

    while let Some(result) = set.join_next().await {
        match result {
            Ok(_) => {}
            Err(e) => tracing::error!("Error in bot execution: {}", e),
        }
    }
}
