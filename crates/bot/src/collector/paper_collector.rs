use hayate_core::traits::{Collector, CollectorStream};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::models::BotEvent;

pub struct PaperCollector {
    rx: broadcast::Receiver<BotEvent>,
}

#[async_trait::async_trait]
impl Collector<BotEvent> for PaperCollector {
    async fn get_event_stream(&self) -> anyhow::Result<CollectorStream<'_, BotEvent>> {
        let rx = self.rx.resubscribe();
        let stream = BroadcastStream::new(rx).filter_map(|r| r.ok());

        Ok(Box::pin(stream))
    }
}

impl PaperCollector {
    pub fn new(rx: broadcast::Receiver<BotEvent>) -> Self {
        Self { rx }
    }
}
