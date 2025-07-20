use hayate_core::traits::{Collector, State};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::StreamExt;

use crate::{
    models::{Fill, InternalEvent, Order, PlaceOrder, Side},
    paper_trade::types::PaperExchangeMessage,
    state::{OrderBookState, PendingOrdersState, PositionState},
};

/// PaperExchange simulates an exchange for paper trading.
/// Currently we assume the bot's trade are small enough to not affect the order book,
/// This is because we rely on external events to update the order book, creating different
/// states locally can lead to data inconsistencies which impacts paper trade accuracy.
#[derive(Debug)]
pub struct PaperExchange {
    /// Channel for broadcasting events internally
    broadcaster: broadcast::Sender<InternalEvent>,
    orderbook: OrderBookState,
    bot_position: PositionState,
    pending_orders: PendingOrdersState,
    next_oid: usize, // Order ID counter
}

impl PaperExchange {
    pub fn new() -> Self {
        let (broadcaster, _) = broadcast::channel(1024);

        Self {
            broadcaster,
            orderbook: OrderBookState::new(1024), // TODO: remove hardcode
            bot_position: PositionState::new(),
            pending_orders: PendingOrdersState::new(),
            next_oid: 1,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InternalEvent> {
        self.broadcaster.subscribe()
    }

    pub async fn run(
        &mut self,
        collector: impl Collector<InternalEvent>,
        mut msg_rx: mpsc::UnboundedReceiver<PaperExchangeMessage>,
    ) -> anyhow::Result<()> {
        let mut source_stream = collector.get_event_stream().await?;

        loop {
            tokio::select! {
                Some(event) = source_stream.next() => {
                    self.process_event(event)?;
                }
                Some(msg) = msg_rx.recv() => {
                    self.process_msg(msg)?;
                }
            }
        }
    }

    pub async fn run_with_shutdown(
        &mut self,
        collector: impl Collector<InternalEvent>,
        mut msg_rx: mpsc::UnboundedReceiver<PaperExchangeMessage>,
        shutdown: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<()> {
        let mut source_stream = collector.get_event_stream().await?;

        loop {
            tokio::select! {
                Some(event) = source_stream.next() => {
                    self.process_event(event)?;
                }
                Some(msg) = msg_rx.recv() => {
                    self.process_msg(msg)?;
                }
                _ = shutdown.cancelled() => {
                    tracing::info!("Shutdown signal received, stopping PaperExchange.");
                    break;
                }
            }
        }

        tracing::info!("PaperExchange has finished running.");
        let summary = self.produce_summary()?;
        tracing::info!("PaperTrade Summary:\n{}", summary);
        Ok(())
    }

    fn process_event(&mut self, event: InternalEvent) -> anyhow::Result<()> {
        self.orderbook.process_event(event.clone())?;
        self.broadcaster.send(event)?;

        let pending_order_fills = self.simulate_pending_order_fills();
        for fill in pending_order_fills {
            let fill_event = InternalEvent::OrderFilled(fill);
            self.bot_position.process_event(fill_event.clone())?;
            self.pending_orders.process_event(fill_event.clone())?;
            self.broadcaster.send(fill_event)?;
        }

        Ok(())
    }

    fn process_msg(&mut self, msg: PaperExchangeMessage) -> anyhow::Result<()> {
        match msg {
            PaperExchangeMessage::PlaceOrder(action) => {
                tracing::info!("Bot order received: {:?}", action);
                self.process_place_order(action)?;
            }
            PaperExchangeMessage::CancelOrder(cancel) => {
                tracing::info!("Bot order received: {:?}", cancel);
                if let Some(order) = self.pending_orders.cancel_order(cancel.oid) {
                    self.broadcaster
                        .send(InternalEvent::OrderCancelled(order))?;
                } else {
                    return Err(anyhow::anyhow!(
                        "Order with OID {} not found for cancellation",
                        cancel.oid
                    ));
                }
            }
            PaperExchangeMessage::Close => {
                // TODO: shutdown
            }
        }

        Ok(())
    }

    fn process_place_order(&mut self, action: PlaceOrder) -> anyhow::Result<()> {
        let order = Order {
            oid: self.next_oid,
            symbol: action.symbol.clone(),
            price: action.price,
            size: action.size,
            side: action.side,
        };
        self.next_oid += 1;

        // Simulate the fills
        let fills = self.simulate_fills(&order, false);

        // Update the pending orders state with the new order and broadcast the event
        let place_order_event = InternalEvent::OrderPlaced(order);
        self.pending_orders
            .process_event(place_order_event.clone())?;
        self.broadcaster.send(place_order_event)?;

        // Update the bot position state and pending order state with the fills and broadcast the events
        for fill in fills {
            let fill_event = InternalEvent::OrderFilled(fill);
            self.bot_position.process_event(fill_event.clone())?;
            self.pending_orders.process_event(fill_event.clone())?;
            self.broadcaster.send(fill_event)?;
        }

        Ok(())
    }

    fn simulate_pending_order_fills(&self) -> Vec<Fill> {
        let mut fills = Vec::new();
        let pending_orders = self.pending_orders.get_inner();

        if let Some(best_bid) = self.orderbook.get_inner().best_bid() {
            for pending_ask in pending_orders.asks_iter() {
                if pending_ask.price > best_bid {
                    break; // No more pending asks can be filled
                }

                fills.extend(self.simulate_fills(pending_ask, true));
            }
        }

        if let Some(best_ask) = self.orderbook.get_inner().best_ask() {
            for pending_bid in pending_orders.bids_iter() {
                if pending_bid.price < best_ask {
                    break; // No more pending bids can be filled
                }

                fills.extend(self.simulate_fills(pending_bid, true));
            }
        }

        fills
    }

    fn simulate_fills(&self, order: &Order, is_maker: bool) -> Vec<Fill> {
        let inner = self.orderbook.get_inner();
        let (fills, _) = match order.side {
            Side::Bid => inner.simulate_buy(order.price, order.size),
            Side::Ask => inner.simulate_sell(order.price, order.size),
        };

        let timestamp = chrono::Utc::now().timestamp_millis() as u64;

        fills
            .into_iter()
            .map(|(price, size)| Fill {
                oid: order.oid,
                side: order.side,
                price,
                size,
                is_maker,
                timestamp,
            })
            .collect::<Vec<_>>()
    }

    fn produce_summary(&self) -> anyhow::Result<String> {
        tracing::debug!("Final Paper Exchange State: {:?}", self);
        let final_price = self.orderbook.get_mid_price().ok_or_else(|| {
            anyhow::anyhow!("Cannot produce paper trade summary: orderbook price not available")
        })?;

        let mut summary = String::new();
        summary.push_str("📊 PAPER TRADING SUMMARY\n");
        summary.push_str(&format!("💰 Current Market Price: {}\n", final_price));
        summary.push_str("🟢 Pending Orders:\n");
        for order in self.pending_orders.get_inner().iter() {
            summary.push_str(&format!(
                "=> Side: {}, Price: {}, Size: {:?}\n",
                order.side, order.price, order.size
            ));
        }
        summary.push_str("🎯 Bot Position:\n");
        summary.push_str(&format!(
            "=> Side: {}, Size: {}, Entry Price: {}\n",
            self.bot_position.get_inner().side,
            self.bot_position.get_inner().size,
            self.bot_position.get_inner().entry_price
        ));
        summary.push_str(&format!(
            "=> Current Value: {}\n",
            self.bot_position.get_inner().current_value(final_price)
        ));
        summary.push_str(&format!(
            "=> Unrealized PnL: {}\n",
            self.bot_position.get_inner().unrealized_pnl(final_price)
        ));

        Ok(summary)
    }
}

impl Default for PaperExchange {
    fn default() -> Self {
        Self::new()
    }
}
