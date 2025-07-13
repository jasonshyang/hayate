use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{broadcast, RwLock},
    task::JoinSet,
};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::traits::{Bot, Collector, Executor, Input, State};

pub fn run_bot<B, S, E, A, I>(
    bot: B,
    states: Vec<Arc<RwLock<S>>>,
    collectors: Vec<Box<dyn Collector<E>>>,
    executor: Vec<Box<dyn Executor<A>>>,
    shutdown: CancellationToken,
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
        let shutdown_signal = shutdown.clone();

        set.spawn(async move {
            tracing::info!("Starting Executor...");
            loop {
                tokio::select! {
                    action = action_rx.recv() => match action {
                        Ok(action) => {
                            match exec.execute(action).await {
                                Ok(_) => tracing::debug!("Action executed successfully."),
                                Err(e) => tracing::error!("Error executing action: {}", e),
                            }
                        }
                        Err(_) => {
                            tracing::info!("Action channel closed, stopping executor.");
                            break;
                        },
                    },
                    _ = shutdown_signal.cancelled() => {
                        tracing::info!("Shutdown signal received, stopping Executor.");
                        break;
                    }
                }
            }
            tracing::info!("Executor finished.");
        });
    }

    // Start the states
    for state in states {
        let mut event_rx = event_tx.subscribe();
        let shutdown_signal = shutdown.clone();

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
                            match state_lock.process_event(event.clone()) {
                                Ok(_) => tracing::debug!("Event processed successfully in state {}:", state_lock.name()),
                                Err(e) => tracing::error!("Error processing event in state {}: {}", state_lock.name(), e),
                            }
                            drop(state_lock);
                        }
                        Err(_) => {
                            tracing::info!("Event channel closed, stopping state.");
                            break;
                        },
                    },
                    _ = shutdown_signal.cancelled() => {
                        tracing::info!("Shutdown signal received, stopping State.");
                        break;
                    }
                }
            }
            tracing::info!("State finished.");
        });
    }

    // Start bot
    let shutdown_signal = shutdown.clone();
    set.spawn(async move {
        tracing::info!("Starting Bot...");
        let mut interval = tokio::time::interval(Duration::from_millis(bot.interval_ms()));

        'bot: loop {
            tokio::select! {
                _ = interval.tick() => {
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
                                match action_tx.send(action) {
                                    Ok(_) => tracing::debug!("Action sent successfully."),
                                    Err(_) => break 'bot,
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error evaluating bot: {}", e);
                            continue;
                        }
                    }
                }
                _ = shutdown_signal.cancelled() => {
                    tracing::info!("Shutdown signal received, stopping Bot.");
                    break;
                }
            }
        }

        tracing::info!("Bot finished.");
    });

    // Start the collectors
    for collector in collectors {
        let sender = event_tx.clone();
        let shutdown_signal = shutdown.clone();

        set.spawn(async move {
            tracing::info!("Starting Collector...");
            let mut event_stream = collector.get_event_stream().await.unwrap();
            loop {
                tokio::select! {
                    Some(event) = event_stream.next() => {
                        match sender.send(event) {
                            Ok(_) => {},
                            Err(_) => break,
                        }
                    }
                    _ = shutdown_signal.cancelled() => {
                        tracing::info!("Shutdown signal received, stopping Collector.");
                        break;
                    }
                }
            }
            tracing::info!("Collector finished.");
        });
    }

    set
}
