use serde::Deserialize;

pub const BYBIT_ENDPOINT: &str = "wss://stream.bybit.com/v5/public/spot";
pub type BybitOrderEntry = Vec<String>; // [price, size]

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum BybitMessage {
    SubscriptionAck {
        success: bool,
        #[serde(rename = "ret_msg")]
        message: String,
        #[serde(rename = "conn_id")]
        connection_id: String,
        #[serde(rename = "req_id")]
        request_id: Option<String>,
        #[serde(rename = "op")]
        operation: String,
    },
    OrderBookUpdate(BybitOrderBookUpdate),
    TradeUpdate(BybitTradeUpdate),
}

#[derive(Deserialize, Debug)]
pub struct BybitOrderBookUpdate {
    /// Topic name
    pub topic: String,
    /// The timestamp (ms) that the system generates the data
    #[serde(rename = "ts")]
    pub timestamp: u64,
    /// Data type: snapshot,delta
    #[serde(rename = "type")]
    pub data_type: BybitDataType,
    /// Order book data
    #[serde(rename = "data")]
    pub data: BybitOrderBookData,
    /// The timestamp from the matching engine when this orderbook data is produced.
    /// It can be correlated with T from public trade channel
    #[serde(rename = "cts")]
    pub correlated_timestamp: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum BybitDataType {
    Snapshot,
    Delta,
}

#[derive(Deserialize, Debug)]
pub struct BybitOrderBookData {
    /// Symbol name, e.g. SOLUSDT_SOL/USDT
    #[serde(rename = "s")]
    pub symbol: String,
    /// Bids. For snapshot stream. Sorted by price in descending order
    /// b[0] is the price, b[1] is the size
    /// The delta data has size=0, which means that all quotations for this price have been filled or cancelled
    #[serde(rename = "b")]
    pub bids: Vec<BybitOrderEntry>,
    /// Asks. For snapshot stream. Sorted by price in ascending order
    #[serde(rename = "a")]
    pub asks: Vec<BybitOrderEntry>,
    /// Update ID, if receive "u"=1, that is a snapshot data due to the restart of the service.
    /// Local orderbook should be reset
    #[serde(rename = "u")]
    pub update_id: u64,
    /// Cross sequence
    #[serde(rename = "seq")]
    pub sequence: u64,
}

#[derive(Deserialize, Debug)]
pub struct BybitTradeUpdate {
    /// Topic name
    pub topic: String,
    /// The timestamp (ms) that the system generates the data
    #[serde(rename = "ts")]
    pub timestamp: u64,
    /// Data type: trade
    #[serde(rename = "type")]
    pub data_type: BybitDataType,
    /// Trade data
    #[serde(rename = "data")]
    pub data: Vec<BybitTradeData>,
}

#[derive(Deserialize, Debug)]
pub struct BybitTradeData {
    /// The timestamp (ms) that the order is filled
    #[serde(rename = "T")]
    pub timestamp: u64,
    /// Symbol name, e.g. SOLUSDT_SOL/USDT
    #[serde(rename = "s")]
    pub symbol: String,
    /// Side of taker
    #[serde(rename = "S")]
    pub side: String,
    /// Trade ID
    #[serde(rename = "v")]
    pub size: String,
    /// Price
    #[serde(rename = "p")]
    pub price: String,
    /// Direction of price change, this is documented but not provided
    // #[serde(rename = "L")]
    // pub direction: String,
    /// Trade ID
    #[serde(rename = "i")]
    pub trade_id: String,
    /// Undocumented on Bybit documentation
    #[serde(rename = "BT")]
    pub bt: bool,
    /// Undocumented on Bybit documentation
    #[serde(rename = "RPI")]
    pub rpi: bool,
}
