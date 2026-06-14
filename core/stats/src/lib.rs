use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use devfee::DevFeeState;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum MiningTargetType {
    User,
    Developer,
}

impl std::fmt::Display for MiningTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiningTargetType::User => write!(f, "User"),
            MiningTargetType::Developer => write!(f, "Developer"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveMiningIdentity {
    pub wallet: String,
    pub worker: String,
    pub target_type: MiningTargetType,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStats {
    pub current_hashrate_hps: f64,
    pub average_hashrate_hps: f64,
    pub candidates_found: u64,
    pub shares_submitted: u64,
    pub pool_accepted_shares: u64,
    pub pool_rejected_shares: u64,
    pub stale_shares: u64,
    pub invalid_shares: u64,
    pub unsupported_submit: u64,
    pub uptime_secs: u64,
    pub job_age_secs: u64,
    pub active_wallet_masked: String,
    pub active_target_type: MiningTargetType,
    pub dev_fee_state: DevFeeState,
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self {
            current_hashrate_hps: 0.0,
            average_hashrate_hps: 0.0,
            candidates_found: 0,
            shares_submitted: 0,
            pool_accepted_shares: 0,
            pool_rejected_shares: 0,
            stale_shares: 0,
            invalid_shares: 0,
            unsupported_submit: 0,
            uptime_secs: 0,
            job_age_secs: 0,
            active_wallet_masked: String::new(),
            active_target_type: MiningTargetType::User,
            dev_fee_state: DevFeeState::Disabled,
        }
    }
}

pub struct StatsManager {
    candidates_found: AtomicU64,
    shares_submitted: AtomicU64,
    pool_accepted_shares: AtomicU64,
    pool_rejected_shares: AtomicU64,
    stale_shares: AtomicU64,
    invalid_shares: AtomicU64,
    unsupported_submit: AtomicU64,

    current_hashrate: Arc<RwLock<f64>>,
    total_hash_samples: AtomicU64,
    total_hash_sum: Arc<RwLock<f64>>,
    active_identity: Arc<RwLock<ActiveMiningIdentity>>,
    dev_fee_state: Arc<RwLock<DevFeeState>>,
    start_time: Instant,
    last_job_time: Arc<RwLock<Instant>>,
}

impl Default for StatsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsManager {
    pub fn new() -> Self {
        Self {
            candidates_found: AtomicU64::new(0),
            shares_submitted: AtomicU64::new(0),
            pool_accepted_shares: AtomicU64::new(0),
            pool_rejected_shares: AtomicU64::new(0),
            stale_shares: AtomicU64::new(0),
            invalid_shares: AtomicU64::new(0),
            unsupported_submit: AtomicU64::new(0),
            current_hashrate: Arc::new(RwLock::new(0.0)),
            total_hash_samples: AtomicU64::new(0),
            total_hash_sum: Arc::new(RwLock::new(0.0)),
            active_identity: Arc::new(RwLock::new(ActiveMiningIdentity {
                wallet: String::new(),
                worker: String::new(),
                target_type: MiningTargetType::User,
            })),
            dev_fee_state: Arc::new(RwLock::new(DevFeeState::Disabled)),
            start_time: Instant::now(),
            last_job_time: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn update_hashrate(&self, hashrate: f64) {
        let mut h = self.current_hashrate.write().await;
        *h = hashrate;

        let mut sum = self.total_hash_sum.write().await;
        *sum += hashrate;
        self.total_hash_samples.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_candidates_found(&self) {
        self.candidates_found.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_shares_submitted(&self) {
        self.shares_submitted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_pool_accepted(&self) {
        self.pool_accepted_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_pool_rejected(&self) {
        self.pool_rejected_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_stale(&self) {
        self.stale_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_invalid(&self) {
        self.invalid_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_unsupported_submit(&self) {
        self.unsupported_submit.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_candidates_found_sync(&self) {
        self.candidates_found.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_pool_accepted_sync(&self) {
        self.pool_accepted_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_pool_rejected_sync(&self) {
        self.pool_rejected_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_stale_sync(&self) {
        self.stale_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn get_stats(&self) -> RuntimeStats {
        let identity = self.active_identity.read().await.clone();

        // Mask the wallet (e.g. keep first 6 and last 4, hide rest)
        let mut masked = String::new();
        if identity.wallet.len() > 10 {
            masked.push_str(&identity.wallet[..6]);
            masked.push_str("...");
            masked.push_str(&identity.wallet[identity.wallet.len() - 4..]);
        } else {
            masked = identity.wallet.clone();
        }

        let current_h = *self.current_hashrate.read().await;
        let samples = self.total_hash_samples.load(Ordering::Relaxed);
        let avg_h = if samples > 0 {
            *self.total_hash_sum.read().await / samples as f64
        } else {
            0.0
        };

        RuntimeStats {
            current_hashrate_hps: current_h,
            average_hashrate_hps: avg_h,
            candidates_found: self.candidates_found.load(Ordering::Relaxed),
            shares_submitted: self.shares_submitted.load(Ordering::Relaxed),
            pool_accepted_shares: self.pool_accepted_shares.load(Ordering::Relaxed),
            pool_rejected_shares: self.pool_rejected_shares.load(Ordering::Relaxed),
            stale_shares: self.stale_shares.load(Ordering::Relaxed),
            invalid_shares: self.invalid_shares.load(Ordering::Relaxed),
            unsupported_submit: self.unsupported_submit.load(Ordering::Relaxed),
            uptime_secs: self.start_time.elapsed().as_secs(),
            active_wallet_masked: masked,
            active_target_type: identity.target_type,
            dev_fee_state: *self.dev_fee_state.read().await,
            job_age_secs: self.last_job_time.read().await.elapsed().as_secs(),
        }
    }

    pub async fn set_identity(&self, identity: ActiveMiningIdentity) {
        let mut w = self.active_identity.write().await;
        *w = identity;
    }

    pub async fn set_dev_fee_state(&self, state: DevFeeState) {
        let mut s = self.dev_fee_state.write().await;
        *s = state;
    }

    pub async fn notify_new_job(&self) {
        let mut last_job = self.last_job_time.write().await;
        *last_job = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_stats_tracking() {
        let manager = StatsManager::new();
        manager.inc_candidates_found();
        manager.inc_pool_accepted();
        manager.inc_pool_rejected();
        manager.inc_stale();
        manager.update_hashrate(1234.5).await;
        manager
            .set_identity(ActiveMiningIdentity {
                wallet: "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(),
                worker: "worker".to_string(),
                target_type: MiningTargetType::User,
            })
            .await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.candidates_found, 1);
        assert_eq!(stats.pool_accepted_shares, 1);
        assert_eq!(stats.pool_rejected_shares, 1);
        assert_eq!(stats.stale_shares, 1);
        assert_eq!(stats.current_hashrate_hps, 1234.5);
        assert_eq!(stats.average_hashrate_hps, 1234.5);
        assert_eq!(stats.active_wallet_masked, "1A1zP1...vfNa");
        assert_eq!(stats.active_target_type, MiningTargetType::User);
    }

    #[tokio::test]
    async fn test_job_age() {
        let manager = StatsManager::new();
        manager.notify_new_job().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let stats = manager.get_stats().await;
        assert!(stats.job_age_secs >= 1);
    }

    #[tokio::test]
    async fn test_concurrent_stats() {
        let manager = Arc::new(StatsManager::new());
        let mut handles = Vec::new();

        for _ in 0..10 {
            let m = manager.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..1000 {
                    m.inc_candidates_found_sync();
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let stats = manager.get_stats().await;
        assert_eq!(stats.candidates_found, 10000);
    }
}
