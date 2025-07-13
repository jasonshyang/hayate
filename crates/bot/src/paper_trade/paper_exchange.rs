use std::collections::BTreeMap;

use hayate_core::traits::{Collector, State};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::StreamExt;

use crate::{
    models::{BotEvent, Decimal, Fill, Side},
    paper_trade::types::PaperExchangeMessage,
    state::{OrderBookState, PositionState},
};

/// PaperExchange simulates an exchange for paper trading.
/// Currently we assume the bot's trade are small enough to not affect the order book,
/// This is because we rely on external events to update the order book, creating different
/// states locally can lead to data inconsistencies which impacts paper trade accuracy.
#[derive(Debug)]
pub struct PaperExchange {
    /// Channel for broadcasting events internally
    broadcaster: broadcast::Sender<BotEvent>,
    /// Queue for unfilled bot orders
    pending_bot_bids: BTreeMap<Decimal, Decimal>,
    pending_bot_asks: BTreeMap<Decimal, Decimal>,
    orderbook: OrderBookState, // TODO: have different book for each symbol
    bot_position: PositionState,
}

impl PaperExchange {
    pub fn new() -> Self {
        let (broadcaster, _) = broadcast::channel(1024);

        Self {
            broadcaster,
            pending_bot_bids: BTreeMap::new(),
            pending_bot_asks: BTreeMap::new(),
            orderbook: OrderBookState::new(1024), // TODO: remove hardcode
            bot_position: PositionState::new(),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BotEvent> {
        self.broadcaster.subscribe()
    }

    pub async fn run(
        &mut self,
        collector: impl Collector<BotEvent>,
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
        collector: impl Collector<BotEvent>,
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

    fn process_event(&mut self, event: BotEvent) -> anyhow::Result<()> {
        self.orderbook.process_event(event.clone())?;
        self.broadcaster.send(event)?;

        if self.is_pending_order_fillable() {
            let fills = self.fill_pending_orders();
            for fill in fills {
                self.bot_position
                    .process_event(BotEvent::OrderFilled(fill.clone()))?;
                self.broadcaster.send(BotEvent::OrderFilled(fill))?;
            }
        }

        Ok(())
    }

    fn process_msg(&mut self, msg: PaperExchangeMessage) -> anyhow::Result<()> {
        match msg {
            PaperExchangeMessage::PlaceOrder(order) => {
                tracing::info!("Bot order received: {:?}", order);
                let side = order.side;
                let price = order.price;
                let size = order.size;

                let (fills, remaining) = self.fill_order(
                    side, price, size, false, // because we are immediately filling the order
                );

                if remaining.is_positive() {
                    // If there are unfilled orders, we add them to the bot pending orders
                    match side {
                        Side::Bid => self.pending_bot_bids.insert(price, size),
                        Side::Ask => self.pending_bot_asks.insert(price, size),
                    };
                }

                for fill in fills {
                    self.bot_position
                        .process_event(BotEvent::OrderFilled(fill.clone()))?;
                    self.broadcaster.send(BotEvent::OrderFilled(fill))?;
                }
            }
            PaperExchangeMessage::CancelOrder(_) => {
                todo!("Cancel order logic not implemented yet");
            }
            PaperExchangeMessage::Close => {
                // TODO: shutdown
            }
        }

        Ok(())
    }

    fn is_pending_order_fillable(&self) -> bool {
        if self.pending_bot_bids.is_empty() || self.pending_bot_asks.is_empty() {
            return false;
        }

        if let Some(best_bid) = self.orderbook.get_inner().best_bid() {
            // Check if the best bid crosses the best ask in pending orders
            if let Some(best_ask) = self.pending_bot_asks.keys().next() {
                if best_bid >= *best_ask {
                    return true;
                }
            }
        }

        if let Some(best_ask) = self.orderbook.get_inner().best_ask() {
            // Check if the best ask crosses the best bid in pending orders
            if let Some(best_bid) = self.pending_bot_bids.keys().next() {
                if best_ask <= *best_bid {
                    return true;
                }
            }
        }

        false
    }

    fn fill_pending_orders(&mut self) -> Vec<Fill> {
        let mut fills = Vec::new();

        // Fill bot's pending bids
        while let Some((bid_price, bid_size)) = self.pending_bot_bids.pop_last() {
            let (bid_fills, remaining_bid) = self.fill_order(Side::Bid, bid_price, bid_size, true);
            fills.extend(bid_fills);

            // If there are remaining size, put it back to pending orders
            if remaining_bid.is_positive() {
                self.pending_bot_bids.insert(bid_price, remaining_bid);
                // Because we are filling from best to worst, we can break early
                break;
            }
        }

        // Fill bot's pending asks
        while let Some((ask_price, ask_size)) = self.pending_bot_asks.pop_first() {
            let (ask_fills, remaining_ask) = self.fill_order(Side::Ask, ask_price, ask_size, true);
            fills.extend(ask_fills);

            // If there are remaining size, put it back to pending orders
            if remaining_ask.is_positive() {
                self.pending_bot_asks.insert(ask_price, remaining_ask);
                // Because we are filling from best to worst, we can break early
                break;
            }
        }

        fills
    }

    fn fill_order(
        &self,
        side: Side,
        price: Decimal,
        size: Decimal,
        is_maker: bool,
    ) -> (Vec<Fill>, Decimal) {
        let inner = self.orderbook.get_inner();
        let (fills, remaining_size) = match side {
            Side::Bid => inner.simulate_buy(price, size),
            Side::Ask => inner.simulate_sell(price, size),
        };

        let timestamp = chrono::Utc::now().timestamp_millis() as u64;

        let fills = fills
            .into_iter()
            .map(|(price, size)| Fill {
                side,
                price,
                size,
                is_maker,
                timestamp,
            })
            .collect::<Vec<_>>();

        (fills, remaining_size)
    }

    fn produce_summary(&self) -> anyhow::Result<String> {
        tracing::debug!("Final Paper Exchange State: {:?}", self);
        let final_price = self.orderbook.get_mid_price().ok_or_else(|| {
            anyhow::anyhow!("Cannot produce paper trade summary: orderbook price not available")
        })?;

        let mut summary = String::new();
        summary.push_str("📊 PAPER TRADING SUMMARY\n");
        summary.push_str(&format!("💰 Current Market Price: {}\n", final_price));
        summary.push_str("🟢 Pending Bids:\n");
        for (price, size) in &self.pending_bot_bids {
            summary.push_str(&format!("=> Price: {}, Size: {}\n", price, size));
        }
        summary.push_str("🔴 Pending Asks:\n");
        for (price, size) in &self.pending_bot_asks {
            summary.push_str(&format!("=> Price: {}, Size: {}\n", price, size));
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
