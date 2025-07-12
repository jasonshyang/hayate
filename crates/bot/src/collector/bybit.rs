use clients::{BybitClient, BybitMessage};
use hayate_core::traits::{Collector, CollectorStream};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::models::{BotEvent, OrderBookEvent, OrderBookUpdate, OrderEntry, Side};

pub struct BybitCollector;

#[async_trait::async_trait]
impl Collector<BotEvent> for BybitCollector {
    async fn get_event_stream(&self) -> anyhow::Result<CollectorStream<'_, BotEvent>> {
        let (tx, rx) = mpsc::unbounded_channel::<BybitMessage>();
        let mut client = BybitClient::new(tx);

        tokio::spawn(async move {
            if let Err(e) = client.connect().await {
                tracing::error!("Failed to connect to Bybit WebSocket: {}", e);
            }
        });

        let stream =
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx).filter_map(|msg| match msg {
                BybitMessage::OrderBookUpdate(update) => {
                    let bids: Vec<OrderEntry> = update
                        .data
                        .bids
                        .into_iter()
                        .filter_map(|mut entry| {
                            // [price, size]
                            let size = entry.pop()?;
                            let price = entry.pop()?;
                            OrderEntry::try_new(Side::Bid, price, size).ok()
                        })
                        .collect();

                    let asks: Vec<OrderEntry> = update
                        .data
                        .asks
                        .into_iter()
                        .filter_map(|mut entry| {
                            // [price, size]
                            let size = entry.pop()?;
                            let price = entry.pop()?;
                            OrderEntry::try_new(Side::Ask, price, size).ok()
                        })
                        .collect();

                    // TODO: handle snapshot and delta separately (perhaps?)
                    let event = OrderBookEvent::Delta(OrderBookUpdate {
                        symbol: update.data.symbol,
                        updated_at: update.timestamp,
                        bids,
                        asks,
                    });
                    Some(BotEvent::OrderBookEvent(event))
                }
                _ => None,
            });
        Ok(Box::pin(stream))
    }
}
