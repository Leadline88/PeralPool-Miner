use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait MiningAlgorithm: Send + Sync {
    type Job: Send + Sync;
    type WorkPackage: Send + Sync;
    type ShareCandidate: Send + Sync;

    fn parse_job(&self, data: &str) -> Result<Self::Job, String>;
    fn create_work(&self, job: &Self::Job, range: NonceRange) -> Self::WorkPackage;
    fn verify_share(&self, job: &Self::Job, share: &Self::ShareCandidate) -> bool;
}

#[async_trait]
pub trait MiningBackend: Send + Sync {
    async fn start(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn set_job(&self, job_data: &str) -> Result<(), String>;
    async fn get_hashrate(&self) -> f64 {
        0.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceRange {
    pub start: u64,
    pub end: u64,
}
