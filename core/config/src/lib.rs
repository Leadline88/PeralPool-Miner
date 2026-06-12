use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub wallet: String,
    pub worker_name: String,
    pub pool_url: String,
    pub mode: MiningMode,
    pub backend: MiningBackend,
    pub algo: String,
    pub miner_binary_path: String,
    pub args: Vec<String>,
    pub threads: usize,
    pub deterministic: bool,
    pub benchmark: bool,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MiningMode {
    Compatibility,
    NativeCpu,
    NativeGpu,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MiningBackend {
    Cpu,
    Cuda,
    Hip,
    Opencl,
    Sycl,
    Metal,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallet: String::new(),
            worker_name: "worker1".to_string(),
            pool_url: String::new(),
            mode: MiningMode::NativeCpu,
            backend: MiningBackend::Cpu,
            algo: "pearl".to_string(),
            miner_binary_path: String::new(),
            args: Vec::new(),
            threads: 0,
            deterministic: false,
            benchmark: false,
            dry_run: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.wallet.is_empty() {
            return Err(ConfigError::Validation(
                "Wallet cannot be empty".to_string(),
            ));
        }
        if self.worker_name.is_empty() {
            return Err(ConfigError::Validation(
                "Worker name cannot be empty".to_string(),
            ));
        }
        if self.pool_url.is_empty() {
            return Err(ConfigError::Validation(
                "Pool URL cannot be empty".to_string(),
            ));
        }
        if self.mode == MiningMode::Compatibility && self.miner_binary_path.is_empty() {
            return Err(ConfigError::Validation(
                "Miner binary path cannot be empty in compatibility mode".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let toml_str = r#"
            wallet = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
            worker_name = "worker1"
            pool_url = "stratum+tcp://pearlpool.cloud:5566"
            mode = "native-cpu"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = "/usr/bin/miner"
            args = ["--algo", "pearl", "--pool", "stratum+tcp://pearlpool.cloud:5566"]
            threads = 0
            deterministic = false
            benchmark = false
            dry_run = false
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config_empty_wallet() {
        let toml_str = r#"
            wallet = ""
            worker_name = "worker1"
            pool_url = "stratum+tcp://pearlpool.cloud:5566"
            mode = "native-cpu"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = "/usr/bin/miner"
            args = []
            threads = 0
            deterministic = false
            benchmark = false
            dry_run = false
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_config_empty_worker_name() {
        let toml_str = r#"
            wallet = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
            worker_name = ""
            pool_url = "stratum+tcp://pearlpool.cloud:5566"
            mode = "native-cpu"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = "/usr/bin/miner"
            args = []
            threads = 0
            deterministic = false
            benchmark = false
            dry_run = false
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_config_empty_pool_url() {
        let toml_str = r#"
            wallet = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
            worker_name = "worker1"
            pool_url = ""
            mode = "native-cpu"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = "/usr/bin/miner"
            args = []
            threads = 0
            deterministic = false
            benchmark = false
            dry_run = false
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_err());
    }
}
