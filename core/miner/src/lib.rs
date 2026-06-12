use std::fmt::Debug;
use async_trait::async_trait;

#[async_trait]
pub trait MiningAlgorithm: Send + Sync + Debug {
    fn name(&self) -> &str;
}

#[async_trait]
pub trait MiningBackend: Send + Sync + Debug {
    fn name(&self) -> &str;
    async fn list_devices(&self) -> Vec<Box<dyn MiningDevice>>;
}

#[async_trait]
pub trait MiningDevice: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn memory_info(&self) -> String;
}

#[async_trait]
pub trait WorkProvider: Send + Sync + Debug {
    async fn get_work(&self) -> Option<Vec<u8>>;
}

#[async_trait]
pub trait ShareSubmitter: Send + Sync + Debug {
    async fn submit_share(&self, share: Vec<u8>) -> bool;
}

pub trait DevFeePolicy: Send + Sync + Debug {
    fn fee_percentage(&self) -> f64;
    fn dev_wallet(&self) -> &str;
}

#[async_trait]
pub trait BenchmarkRunner: Send + Sync + Debug {
    async fn run_benchmark(&self) -> Result<f64, String>;
}
