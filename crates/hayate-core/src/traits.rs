use std::{pin::Pin, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;
use tokio_stream::Stream;

pub type CollectorStream<'a, E> = Pin<Box<dyn Stream<Item = E> + Send + 'a>>;

#[async_trait::async_trait]
pub trait Collector<E>: Send + Sync {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, E>>;
}

#[async_trait::async_trait]
pub trait State<E>: Send + Sync {
    async fn sync(&mut self) -> Result<()>;
    fn name(&self) -> &str;
    fn process_event(&mut self, event: E) -> Result<()>;
}

pub trait Bot<S, A>: Send + Sync {
    fn new(states: impl IntoIterator<Item = Arc<RwLock<S>>>) -> Self;
    fn interval(&self) -> u64;
    fn evaluate(&self) -> Result<Vec<A>>;
}

#[async_trait::async_trait]
pub trait Executor<A>: Send + Sync {
    async fn execute(&self, action: A) -> Result<()>;
}
