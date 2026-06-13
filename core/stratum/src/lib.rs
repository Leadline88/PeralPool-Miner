pub mod adapter;
pub mod client;
pub mod miner_loop;
pub mod mock_adapter;
pub mod mock_server;
pub mod protocol;

#[cfg(test)]
mod tests;

pub use adapter::PoolAdapter;
pub use client::StratumClient;
pub use protocol::{JsonRpcRequest, JsonRpcResponse, StratumMessage};
