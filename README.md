# Hayate

**Hayate** (疾風) is an algorithmic trading bot framework built in Rust, providing a clean architecture for building trading bots with real-time market data processing, state management, and execution capabilities.

## Bot Strategies

Currently only two strategies are available as examples:
- Simple Market Making: place limit order on both sides of the orderbook with a fixed spread
- Dynamic Spread Market Making; place limit order on both sides of the orderbook with a dynamic spread based on RSI and NATR

## Development Status

Hayate is currently in active development. Core features are functional but the API may change.

**Current Status**:
- ✅ Core framework architecture
- ✅ Bybit integration
- ✅ OrderBook and Position state management  
- ✅ Simple market making strategy
- ✅ Paper trade simulator
- 🚧 Additional state management
- 🚧 Multiple trading pairs
- 🚧 Additional exchange integrations
- 🚧 Advanced trading strategies

## Architecture Overview

```
External Sources                    External Targets
(Exchanges, Feeds)                 (Exchanges, APIs)
        │                                  ▲
        │ Market Data                      │ Orders
        ▼                                  │
┌───────────────┐                 ┌───────────────┐
│  Collector<E> │                 │  Executor<A>  │
│               │                 │               │
│ Stream Events │                 │ Execute       │
└───────┬───────┘                 └───────▲───────┘
        │                                 │
        │ Events (E)                      │ Actions (A)
        ▼                                 │
┌───────────────┐                 ┌───────────────┐
│   State<E>    │    Input<S>     │   Bot<I,A>    │
│               │◄────────────────│               │
│ Maintain      │                 │ Strategy      │
│ Internal      │                 │ Logic         │
│ State         │                 │               │
└───────────────┘                 └───────────────┘
```

## Core Components

`hayate-core` provides the following traits representing the core components needed for the Hayate Bot:

* **`Collector<E>`**: Responsible for providing a stream of events from external sources (exchanges, data feeds)
* **`State<E>`**: Maintains internal state (e.g. orderbook, positions) based on incoming events `E`
* **`Bot<I, A>`**: Consumes input `I` and outputs a list of actions `A` based on the bot's trading strategy
* **`Executor<A>`**: Responsible for executing actions `A` (e.g. submit orders to exchange)
* **`Input<S>`**: Connects `State` and `Bot` together, takes a reference to state `S` and modifies self

## Crates Overview

### 📦 `hayate-core`
The core crate containing traits with a `run_bot` function to orchestrate the entire system.

### 🤖 `bot`
Contains bot implementations, trading models, and business logic. Includes:
- **Collectors**: Data ingestion from exchanges (e.g. `BybitCollector`)
- **States**: State management (e.g. `OrderBookState`, `PositionState`, `PriceState`)  
- **Core**: Trading strategies (e.g. `SimpleMarketMaking`)
- **Executors**: Trade execution
- **Models**: Data structures and types used throughout the system

#### 📄 Paper Trading
The bot crate includes a comprehensive paper trading system for testing and validation:

- **`PaperExchange`**: Simulates a real exchange environment with order matching and fills
- **`PaperCollector`**: Collects events from the paper exchange for bot consumption  
- **`PaperExecutor`**: Executes bot actions within the simulated environment

The paper exchange acts as a proxy, taking real market data from any source collector that implements `Collector<InternalEvent>` and simulating trade execution against that live data. This allows you to:

- **Test strategies** with real market conditions without risking capital  
- **Validate bot logic** before deploying to live trading  
- **Analyze performance** with detailed trade simulation and P&L tracking  
- **Switch data sources** easily by plugging in different collectors

### 🔗 `clients`
Exchange-specific client implementations for connecting to trading platforms. Currently supports Bybit WebSocket API with plans for additional exchanges.

### 🌐 `transport`
Networking layer providing HTTP and WebSocket client abstractions. Handles connection management, reconnection logic, and message parsing.

## Usage

### Quick Start
```bash
# Run the simple market making bot with paper trade
cargo run --bin simple_market_making

# Run the dynamic spread market making bot with paper trade
cargo run --bin market_making_with_dynamic_spread
```