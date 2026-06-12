use serde_json::{json, Value};
use shares::ShareCandidate;
use stratum::{JsonRpcRequest, JsonRpcResponse, PoolAdapter};

pub const DEFAULT_POOL_URL: &str = "stratum+tcp://pearlpool.cloud:5566";
pub const DEFAULT_ALGO: &str = "pearl";

pub struct PearlPoolAdapter;

impl PoolAdapter for PearlPoolAdapter {
    fn name(&self) -> &str {
        "PearlPool"
    }

    fn build_subscribe_request(&self) -> JsonRpcRequest {
        JsonRpcRequest {
            id: None,
            method: "mining.subscribe".to_string(),
            params: json!(["pearl-miner/0.1.0"]),
        }
    }

    fn parse_subscribe_response(&self, response: &JsonRpcResponse) -> Result<(), String> {
        if let Some(error) = &response.error {
            return Err(error.to_string());
        }
        Ok(())
    }

    fn build_login_request(&self, wallet: &str, worker: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            id: None, // Will be set by client
            method: "mining.authorize".to_string(),
            params: json!([wallet, worker]),
        }
    }

    fn parse_login_response(&self, response: &JsonRpcResponse) -> Result<bool, String> {
        if let Some(error) = &response.error {
            return Err(error.to_string());
        }
        match &response.result {
            Some(Value::Bool(b)) => Ok(*b),
            _ => Err("Invalid login response".to_string()),
        }
    }

    fn parse_job(&self, request: &JsonRpcRequest) -> Option<Value> {
        if request.method == "mining.notify" {
            Some(request.params.clone())
        } else {
            None
        }
    }

    fn parse_difficulty(&self, request: &JsonRpcRequest) -> Option<f64> {
        if request.method == "mining.set_difficulty" {
            request.params.as_array()?.first()?.as_f64()
        } else {
            None
        }
    }

    fn build_share_submit(&self, share: &ShareCandidate) -> JsonRpcRequest {
        JsonRpcRequest {
            id: None,
            method: "mining.submit".to_string(),
            params: json!([
                share.worker,
                share.job_id,
                "0x00000000",
                share.timestamp.to_rfc3339(),
                share.nonce
            ]),
        }
    }

    fn parse_share_response(&self, response: &JsonRpcResponse) -> Result<bool, String> {
        if let Some(error) = &response.error {
            return Err(error.to_string());
        }
        match &response.result {
            Some(Value::Bool(b)) => Ok(*b),
            _ => Err("Invalid share submission response".to_string()),
        }
    }
}

pub fn get_default_profile() -> String {
    format!("Pool URL: {}, Algo: {}", DEFAULT_POOL_URL, DEFAULT_ALGO)
}
