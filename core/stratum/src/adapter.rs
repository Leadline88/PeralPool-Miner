use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;
use shares::ShareCandidate;

pub trait PoolAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn build_subscribe_request(&self) -> JsonRpcRequest;
    fn parse_subscribe_response(&self, response: &JsonRpcResponse) -> Result<(), String>;

    fn build_login_request(&self, wallet: &str, worker: &str) -> JsonRpcRequest;
    fn parse_login_response(&self, response: &JsonRpcResponse) -> Result<bool, String>;

    fn parse_job(&self, request: &JsonRpcRequest) -> Option<Value>;
    fn parse_difficulty(&self, request: &JsonRpcRequest) -> Option<f64>;

    fn build_share_submit(&self, share: &ShareCandidate) -> JsonRpcRequest;
    fn parse_share_response(&self, response: &JsonRpcResponse) -> Result<bool, String>;
}
