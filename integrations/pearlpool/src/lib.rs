use serde_json::{json, Value};
use shares::ShareCandidate;
use stratum::{JsonRpcRequest, JsonRpcResponse, PoolAdapter};

pub const DEFAULT_POOL_URL: &str = "stratum+tcp://pearlpool.cloud:5566";
pub const DEFAULT_PORT: u16 = 5566;
pub const DEFAULT_ALGO: &str = "pearl";

pub fn recommended_config() -> &'static str {
    r#"
[profiles.pearlpool]
pool_url = "stratum+tcp://pearlpool.cloud:5566"
algo = "pearl"
miner = "lpminer"

[miners.lpminer]
binary_path = "./miners/lpminer/lpminer.exe"
args = ["--pool", "{pool_url}", "--wallet", "{wallet}", "--worker", "{worker}"]

[miners.srbminer]
binary_path = "./miners/srbminer/SRBMiner-MULTI.exe"
args = ["--algorithm", "pearl", "--pool", "{pool_url}", "--wallet", "{wallet}.{worker}"]
"#
}

pub fn validate_credentials(wallet: &str, worker: &str) -> Result<(), String> {
    if wallet.trim().is_empty() {
        return Err("Wallet address cannot be empty".to_string());
    }
    if worker.trim().is_empty() {
        return Err("Worker name cannot be empty".to_string());
    }
    Ok(())
}

pub struct PearlPoolAdapter {
    pub allow_experimental: bool,
}

impl PearlPoolAdapter {
    pub fn new(allow_experimental: bool) -> Self {
        Self { allow_experimental }
    }
}

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
            if !self.allow_experimental {
                tracing::error!("Live PearlPool native mining is not verified yet; use compatibility mode or synthetic CPU mode.");
                return None;
            }
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
        if !self.allow_experimental {
            // Return a dummy request that will fail verification if somehow called
            return JsonRpcRequest {
                id: None,
                method: "mining.unsupported_real_pearl_share_submit_format".to_string(),
                params: json!([]),
            };
        }
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

pub fn build_lpminer_cmd(pool_url: &str, wallet: &str, worker: &str) -> (String, Vec<String>) {
    (
        "./miners/lpminer/lpminer.exe".to_string(),
        vec![
            "--pool".to_string(),
            pool_url.to_string(),
            "--wallet".to_string(),
            wallet.to_string(),
            "--worker".to_string(),
            worker.to_string(),
        ],
    )
}

pub fn build_srbminer_cmd(pool_url: &str, wallet: &str, worker: &str) -> (String, Vec<String>) {
    (
        "./miners/srbminer/SRBMiner-MULTI.exe".to_string(),
        vec![
            "--algorithm".to_string(),
            "pearl".to_string(),
            "--pool".to_string(),
            pool_url.to_string(),
            "--wallet".to_string(),
            format!("{}.{}", wallet, worker),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_credentials() {
        assert!(validate_credentials("wallet123", "worker1").is_ok());
        assert!(validate_credentials("", "worker1").is_err());
        assert!(validate_credentials("wallet123", "").is_err());
    }

    #[test]
    fn test_build_lpminer_cmd() {
        let (binary, args) =
            build_lpminer_cmd("stratum+tcp://pearlpool.cloud:5566", "1Wallet", "worker1");
        assert_eq!(binary, "./miners/lpminer/lpminer.exe");
        assert_eq!(
            args,
            vec![
                "--pool",
                "stratum+tcp://pearlpool.cloud:5566",
                "--wallet",
                "1Wallet",
                "--worker",
                "worker1"
            ]
        );
    }

    #[test]
    fn test_build_srbminer_cmd() {
        let (binary, args) =
            build_srbminer_cmd("stratum+tcp://pearlpool.cloud:5566", "1Wallet", "worker1");
        assert_eq!(binary, "./miners/srbminer/SRBMiner-MULTI.exe");
        assert_eq!(
            args,
            vec![
                "--algorithm",
                "pearl",
                "--pool",
                "stratum+tcp://pearlpool.cloud:5566",
                "--wallet",
                "1Wallet.worker1"
            ]
        );
    }

    #[test]
    fn test_import_integrity() {
        // This test ensures that PearlPoolAdapter can be instantiated and implements PoolAdapter
        // which verifies the types from 'stratum' and 'shares' resolve correctly.
        let adapter = PearlPoolAdapter::new(true);
        assert_eq!(adapter.name(), "PearlPool");

        let share = ShareCandidate {
            job_id: "test".to_string(),
            nonce: "123".to_string(),
            result: "hash".to_string(),
            worker: "worker".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let req = adapter.build_share_submit(&share);
        assert_eq!(req.method, "mining.submit");
    }

    #[test]
    fn test_gating() {
        let adapter = PearlPoolAdapter::new(false);
        let notify = JsonRpcRequest {
            id: None,
            method: "mining.notify".to_string(),
            params: json!([]),
        };
        assert!(adapter.parse_job(&notify).is_none());
    }
}
