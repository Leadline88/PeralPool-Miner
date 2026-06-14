use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;
use shares::ShareCandidate;

#[derive(Debug, thiserror::Error)]
pub enum PoolAdapterError {
    #[error("Unsupported real Pearl job format")]
    UnsupportedRealPearlJobFormat,
    #[error("Unsupported real Pearl share submit format")]
    UnsupportedRealPearlShareSubmitFormat,
    #[error("Experimental live Stratum disabled")]
    ExperimentalLiveStratumDisabled,
    #[error("Invalid share candidate")]
    InvalidShareCandidate,
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Real Pearl algorithm not implemented")]
    RealPearlAlgorithmNotImplemented,
    #[error("PearlPool live mining not verified")]
    PearlPoolLiveMiningNotVerified,
    #[error("Active dev-fee identity switching not implemented")]
    ActiveDevFeeIdentitySwitchingNotImplemented,
    #[error("GPU backend not implemented")]
    GpuBackendNotImplemented,
}

pub trait PoolAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn build_subscribe_request(&self) -> JsonRpcRequest;
    fn parse_subscribe_response(&self, response: &JsonRpcResponse) -> Result<(), String>;

    fn build_login_request(&self, wallet: &str, worker: &str) -> JsonRpcRequest;
    fn parse_login_response(&self, response: &JsonRpcResponse) -> Result<bool, String>;

    fn parse_job(&self, request: &JsonRpcRequest) -> Option<Value>;
    fn parse_difficulty(&self, request: &JsonRpcRequest) -> Option<f64>;

    fn build_share_submit(
        &self,
        share: &ShareCandidate,
    ) -> Result<JsonRpcRequest, PoolAdapterError>;
    fn parse_share_response(&self, response: &JsonRpcResponse) -> Result<bool, String>;
}
