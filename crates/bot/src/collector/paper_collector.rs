use hayate_core::traits::{Collector, CollectorStream};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::models::InternalEvent;

pub struct PaperCollector {
    rx: broadcast::Receiver<InternalEvent>,
}

#[async_trait::async_trait]
impl Collector<InternalEvent> for PaperCollector {
    async fn get_event_stream(&self) -> anyhow::Result<CollectorStream<'_, InternalEvent>> {
        let rx = self.rx.resubscribe();
        let stream = BroadcastStream::new(rx).filter_map(|r| r.ok());

        Ok(Box::pin(stream))
    }
}

impl PaperCollector {
    pub fn new(rx: broadcast::Receiver<InternalEvent>) -> Self {
        Self { rx }
    }
}
