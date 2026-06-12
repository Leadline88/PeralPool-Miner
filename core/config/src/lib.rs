use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub wallet: String,
    pub worker_name: String,
    pub pool_url: String,
    pub miner_binary_path: String,
    pub args: Vec<String>,
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
        if self.miner_binary_path.is_empty() {
            return Err(ConfigError::Validation(
                "Miner binary path cannot be empty".to_string(),
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
            miner_binary_path = "/usr/bin/miner"
            args = ["--algo", "pearl", "--pool", "stratum+tcp://pearlpool.cloud:5566"]
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
            miner_binary_path = "/usr/bin/miner"
            args = []
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
            miner_binary_path = "/usr/bin/miner"
            args = []
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
            miner_binary_path = "/usr/bin/miner"
            args = []
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_config_empty_miner_binary_path() {
        let toml_str = r#"
            wallet = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
            worker_name = "worker1"
            pool_url = "stratum+tcp://pearlpool.cloud:5566"
            miner_binary_path = ""
            args = []
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_err());
    }
}
