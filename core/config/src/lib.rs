use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Profile {
    pub pool_url: String,
    pub algo: String,
    pub miner: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MinerConfig {
    pub binary_path: String,
    pub args: Vec<String>,
}

impl MinerConfig {
    pub fn expand_args(
        &self,
        wallet: &str,
        worker: &str,
        pool_url: &str,
        algo: &str,
    ) -> Vec<String> {
        self.args
            .iter()
            .map(|arg| {
                arg.replace("{wallet}", wallet)
                    .replace("{worker}", worker)
                    .replace("{pool_url}", pool_url)
                    .replace("{algo}", algo)
            })
            .collect()
    }
}

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
    pub profiles: Option<HashMap<String, Profile>>,
    pub miners: Option<HashMap<String, MinerConfig>>,
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
            profiles: None,
            miners: None,
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
        match self.mode {
            MiningMode::Compatibility => {
                if self.miner_binary_path.is_empty() {
                    return Err(ConfigError::Validation(
                        "Miner binary path cannot be empty in compatibility mode. Use --miner-binary or set it in config.".to_string(),
                    ));
                }
            }
            MiningMode::NativeCpu => {
                // native-cpu offline mode requires nothing mandatory as it uses synthetic defaults
            }
            MiningMode::NativeGpu => {
                // Validation for future GPU parameters can go here, but CLI currently returns NotImplemented
            }
        }
        Ok(())
    }

    /// Full validation including network requirements (used for experimental live Stratum)
    pub fn validate_live(&self) -> Result<(), ConfigError> {
        if self.wallet.is_empty() {
            return Err(ConfigError::Validation(
                "Wallet cannot be empty for experimental live stratum mining. Use --wallet or set it in config.".to_string(),
            ));
        }
        if self.worker_name.is_empty() {
            return Err(ConfigError::Validation(
                "Worker name cannot be empty for experimental live stratum mining. Use --worker or set it in config.".to_string(),
            ));
        }
        if self.pool_url.is_empty() {
            return Err(ConfigError::Validation(
                "Pool URL cannot be empty for experimental live stratum mining. Use --pool or set it in config.".to_string(),
            ));
        }
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_expansion() {
        let miner = MinerConfig {
            binary_path: "./miners/testminer".to_string(),
            args: vec![
                "--pool".to_string(),
                "{pool_url}".to_string(),
                "--wallet".to_string(),
                "{wallet}".to_string(),
                "--worker".to_string(),
                "{worker}".to_string(),
                "--algo".to_string(),
                "{algo}".to_string(),
            ],
        };

        let expanded = miner.expand_args(
            "1WalletAddress",
            "worker1",
            "stratum+tcp://pool.com:1234",
            "pearl",
        );

        assert_eq!(
            expanded,
            vec![
                "--pool",
                "stratum+tcp://pool.com:1234",
                "--wallet",
                "1WalletAddress",
                "--worker",
                "worker1",
                "--algo",
                "pearl"
            ]
        );
    }

    #[test]
    fn test_profiles_and_miners_parsing() {
        let toml_str = r#"
            wallet = "1A1z"
            worker_name = "worker1"
            pool_url = "tcp://pool"
            mode = "compatibility"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = ""
            args = []
            threads = 0
            deterministic = false
            benchmark = false
            dry_run = false

            [profiles.pearlpool]
            pool_url = "stratum+tcp://pearlpool.cloud:5566"
            algo = "pearl"
            miner = "lpminer"

            [miners.lpminer]
            binary_path = "./miners/lpminer/lpminer.exe"
            args = ["--pool", "{pool_url}", "--wallet", "{wallet}", "--worker", "{worker}"]
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.profiles.is_some());
        assert!(config.miners.is_some());

        let profiles = config.profiles.unwrap();
        let miners = config.miners.unwrap();

        assert_eq!(profiles.get("pearlpool").unwrap().miner, "lpminer");
        assert_eq!(
            miners.get("lpminer").unwrap().binary_path,
            "./miners/lpminer/lpminer.exe"
        );
    }

    #[test]
    fn test_valid_config_native_cpu_offline() {
        let toml_str = r#"
            wallet = ""
            worker_name = "worker1"
            pool_url = ""
            mode = "native-cpu"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = ""
            args = []
            threads = 0
            deterministic = false
            benchmark = false
            dry_run = false
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_ok());
        assert!(config.validate_live().is_err());
    }

    #[test]
    fn test_valid_config_compatibility() {
        let toml_str = r#"
            wallet = ""
            worker_name = ""
            pool_url = ""
            mode = "compatibility"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = "/usr/bin/miner"
            args = ["--arg1"]
            threads = 0
            deterministic = false
            benchmark = false
            dry_run = false
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config_compatibility_no_binary() {
        let toml_str = r#"
            wallet = ""
            worker_name = ""
            pool_url = ""
            mode = "compatibility"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = ""
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
    fn test_valid_live_config() {
        let toml_str = r#"
            wallet = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
            worker_name = "worker1"
            pool_url = "stratum+tcp://pearlpool.cloud:5566"
            mode = "native-cpu"
            backend = "cpu"
            algo = "pearl"
            miner_binary_path = ""
            args = []
            threads = 0
            deterministic = false
            benchmark = false
            dry_run = false
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate_live().is_ok());
    }
}
