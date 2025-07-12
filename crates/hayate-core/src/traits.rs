use std::pin::Pin;

use anyhow::Result;
use tokio_stream::Stream;

pub type CollectorStream<'a, E> = Pin<Box<dyn Stream<Item = E> + Send + 'a>>;

#[async_trait::async_trait]
pub trait Collector<E>: Send + Sync {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, E>>;
}

#[async_trait::async_trait]
pub trait State<E>: Send + Sync {
    fn name(&self) -> &str;
    async fn sync(&mut self) -> Result<()>;
    fn process_event(&mut self, event: E) -> Result<()>;
}

pub trait Bot<I, A>: Send + Sync {
    fn interval_ms(&self) -> u64;
    fn evaluate(&self, input: I) -> Result<Vec<A>>;
}

#[async_trait::async_trait]
pub trait Executor<A>: Send + Sync {
    async fn execute(&self, action: A) -> Result<()>;
}

pub trait Input<S> {
    fn empty() -> Self;
    fn read_state(&mut self, state: &S) -> Result<()>;
}
