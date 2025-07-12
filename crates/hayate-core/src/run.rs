use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{broadcast, RwLock},
    task::JoinSet,
};
use tokio_stream::StreamExt;

use crate::traits::{Bot, Collector, Executor, Input, State};

pub fn run_bot<B, S, E, A, I>(
    bot: B,
    states: Vec<Arc<RwLock<S>>>,
    collectors: Vec<Box<dyn Collector<E>>>,
    executor: Vec<Box<dyn Executor<A>>>,
) -> JoinSet<()>
where
    B: Bot<I, A> + Send + Sync + 'static,
    S: State<E> + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
    A: Clone + Send + Sync + 'static,
    I: Input<S> + Send + Sync + 'static,
{
    let mut set = JoinSet::new();
    let states_clone = states.clone();

    // Set up bot internal channels
    let (event_tx, _) = broadcast::channel::<E>(1024);
    let (action_tx, _) = broadcast::channel::<A>(1024);

    // Start the executor
    for exec in executor {
        let mut action_rx = action_tx.subscribe();
        set.spawn(async move {
            tracing::info!("Starting Executor...");
            while let Ok(action) = action_rx.recv().await {
                if let Err(e) = exec.execute(action).await {
                    tracing::error!("Error executing action: {}", e);
                    break;
                } else {
                    tracing::debug!("Action executed successfully.");
                }
            }
            tracing::info!("Executor finished.");
        });
    }

    // Start the states
    for state in states {
        let mut event_rx = event_tx.subscribe();

        set.spawn(async move {
            tracing::info!("Starting State");
            let mut state_lock = state.write().await;
            state_lock.sync().await.unwrap();
            tracing::info!("State {} synced.", state_lock.name());
            drop(state_lock);

            loop {
                tokio::select! {
                    event = event_rx.recv() => match event {
                        Ok(event) => {
                            let mut state_lock = state.write().await;
                            if let Err(e) = state_lock.process_event(event.clone()) {
                                tracing::error!("Error processing event: {}", e);
                                break;
                            }
                            drop(state_lock);
                        }
                        Err(_) => {
                            tracing::info!("Event channel closed, stopping state.");
                            break;
                        },
                    },
                }
            }
            tracing::info!("State finished.");
        });
    }

    // Start bot
    set.spawn(async move {
        tracing::info!("Starting Bot...");
        let mut interval = tokio::time::interval(Duration::from_millis(bot.interval_ms()));

        loop {
            interval.tick().await;

            let mut input = I::empty();

            // FIXME: distribute the state reading
            for state in &states_clone {
                let lock = state.read().await;
                if let Err(e) = input.read_state(&*lock) {
                    tracing::error!("Error reading state: {}", e);
                    continue;
                }
                drop(lock);
            }

            match bot.evaluate(input) {
                Ok(actions) => {
                    for action in actions {
                        if let Err(e) = action_tx.send(action) {
                            tracing::error!("Error sending action: {}", e);
                            continue;
                        } else {
                            tracing::debug!("Action sent successfully.");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error evaluating bot: {}", e);
                    continue;
                }
            }
        }
    });

    // Start the collectors
    for collector in collectors {
        let sender = event_tx.clone();
        set.spawn(async move {
            tracing::info!("Starting Collector...");
            let mut event_stream = collector.get_event_stream().await.unwrap();
            while let Some(event) = event_stream.next().await {
                match sender.send(event) {
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
            tracing::info!("Collector finished.");
        });
    }

    set
}
