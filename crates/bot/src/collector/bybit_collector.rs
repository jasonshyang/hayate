use std::str::FromStr;

use clients::{BybitClient, BybitDataType, BybitMessage};
use hayate_core::traits::{Collector, CollectorStream};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::models::{InternalEvent, OrderBookEventKind, OrderBookUpdate, Side, Trade};

pub struct BybitCollector {
    shutdown: CancellationToken,
}

#[async_trait::async_trait]
impl Collector<InternalEvent> for BybitCollector {
    async fn get_event_stream(&self) -> anyhow::Result<CollectorStream<'_, InternalEvent>> {
        let (tx, rx) = mpsc::unbounded_channel::<BybitMessage>();
        let mut client = BybitClient::new_with_shutdown(tx, self.shutdown.clone());

        tokio::spawn(async move {
            if let Err(e) = client.connect().await {
                tracing::error!("Failed to connect to Bybit WebSocket: {}", e);
            }
        });

        let stream =
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx).filter_map(|msg| match msg {
                BybitMessage::OrderBookUpdate(update) => {
                    let bids = update
                        .data
                        .bids
                        .into_iter()
                        .filter_map(|mut entry| {
                            // [price, size]
                            let size = entry.pop()?.try_into().ok()?;
                            let price = entry.pop()?.try_into().ok()?;

                            Some((price, size))
                        })
                        .collect();

                    let asks = update
                        .data
                        .asks
                        .into_iter()
                        .filter_map(|mut entry| {
                            // [price, size]
                            let size = entry.pop()?.try_into().ok()?;
                            let price = entry.pop()?.try_into().ok()?;

                            Some((price, size))
                        })
                        .collect();

                    let kind = match update.data_type {
                        BybitDataType::Snapshot => OrderBookEventKind::Snapshot,
                        BybitDataType::Delta => OrderBookEventKind::Delta,
                    };

                    let data = OrderBookUpdate {
                        symbol: update.data.symbol,
                        kind,
                        updated_at: update.timestamp,
                        bids,
                        asks,
                    };

                    match update.data_type {
                        BybitDataType::Snapshot => Some(InternalEvent::OrderBookUpdate(data)),
                        BybitDataType::Delta => Some(InternalEvent::OrderBookUpdate(data)),
                    }
                }
                BybitMessage::TradeUpdate(update) => {
                    let trades = update
                        .data
                        .into_iter()
                        .filter_map(|trade| {
                            let price = trade.price.try_into().ok()?;
                            let size = trade.size.try_into().ok()?;
                            let timestamp = update.timestamp;

                            Some(Trade {
                                symbol: update.topic.clone(),
                                price,
                                size,
                                side: Side::from_str(&trade.side).ok()?,
                                timestamp,
                            })
                        })
                        .collect::<Vec<_>>();

                    if trades.is_empty() {
                        None
                    } else {
                        Some(InternalEvent::TradeUpdate(trades))
                    }
                }
                _ => None,
            });
        Ok(Box::pin(stream))
    }
}

impl BybitCollector {
    pub fn new(shutdown: CancellationToken) -> Self {
        Self { shutdown }
    }
}
