use crate::adapter::PoolAdapter;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{json, Value};
use shares::ShareCandidate;

pub struct MockPoolAdapter;

impl PoolAdapter for MockPoolAdapter {
    fn name(&self) -> &str {
        "MockPool"
    }

    fn build_subscribe_request(&self) -> JsonRpcRequest {
        JsonRpcRequest {
            id: None,
            method: "mining.subscribe".to_string(),
            params: json!([]),
        }
    }

    fn parse_subscribe_response(&self, _response: &JsonRpcResponse) -> Result<(), String> {
        Ok(())
    }

    fn build_login_request(&self, wallet: &str, worker: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            id: None,
            method: "mining.authorize".to_string(),
            params: json!([wallet, worker]),
        }
    }

    fn parse_login_response(&self, response: &JsonRpcResponse) -> Result<bool, String> {
        match &response.result {
            Some(Value::Bool(b)) => Ok(*b),
            _ => Ok(true),
        }
    }

    fn parse_job(&self, request: &JsonRpcRequest) -> Option<Value> {
        if request.method == "mining.notify" {
            Some(request.params.clone())
        } else {
            None
        }
    }

    fn parse_difficulty(&self, _request: &JsonRpcRequest) -> Option<f64> {
        None
    }

    fn build_share_submit(&self, share: &ShareCandidate) -> JsonRpcRequest {
        JsonRpcRequest {
            id: None,
            method: "mining.submit".to_string(),
            params: json!([share.worker, share.job_id, share.nonce,]),
        }
    }

    fn parse_share_response(&self, _response: &JsonRpcResponse) -> Result<bool, String> {
        Ok(true)
    }
}
