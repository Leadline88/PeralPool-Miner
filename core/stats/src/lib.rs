use mining::{ActiveMiningIdentity, DevFeeState, MiningTargetType};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStats {
    pub hashrate: f64,
    pub candidates_found: u64,
    pub shares_submitted: u64,
    pub pool_accepted_shares: u64,
    pub pool_rejected_shares: u64,
    pub stale_shares: u64,
    pub invalid_shares: u64,
    pub uptime_secs: u64,
    pub current_wallet: String,
    pub active_wallet_masked: String,
    pub active_target_type: MiningTargetType,
    pub dev_fee_state: DevFeeState,
    pub job_age_secs: u64,
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self {
            hashrate: 0.0,
            candidates_found: 0,
            shares_submitted: 0,
            pool_accepted_shares: 0,
            pool_rejected_shares: 0,
            stale_shares: 0,
            invalid_shares: 0,
            uptime_secs: 0,
            current_wallet: String::new(),
            active_wallet_masked: String::new(),
            active_target_type: MiningTargetType::User,
            dev_fee_state: DevFeeState::Disabled,
            job_age_secs: 0,
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

    hashrate: Arc<RwLock<f64>>,
    current_identity: Arc<RwLock<ActiveMiningIdentity>>,
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
            hashrate: Arc::new(RwLock::new(0.0)),
            current_identity: Arc::new(RwLock::new(ActiveMiningIdentity {
                wallet: String::new(),
                worker: String::new(),
                target_type: MiningTargetType::User,
            })),
            dev_fee_state: Arc::new(RwLock::new(DevFeeState::Disabled)),
            start_time: Instant::now(),
            last_job_time: Arc::new(RwLock::new(Instant::now())),
        }
    }

    fn mask_wallet(wallet: &str) -> String {
        if wallet.len() <= 10 {
            return wallet.to_string();
        }
        format!("{}...{}", &wallet[..6], &wallet[wallet.len() - 4..])
    }

    pub async fn update_hashrate(&self, hashrate: f64) {
        let mut h = self.hashrate.write().await;
        *h = hashrate;
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
        let identity = self.current_identity.read().await;
        RuntimeStats {
            hashrate: *self.hashrate.read().await,
            candidates_found: self.candidates_found.load(Ordering::Relaxed),
            shares_submitted: self.shares_submitted.load(Ordering::Relaxed),
            pool_accepted_shares: self.pool_accepted_shares.load(Ordering::Relaxed),
            pool_rejected_shares: self.pool_rejected_shares.load(Ordering::Relaxed),
            stale_shares: self.stale_shares.load(Ordering::Relaxed),
            invalid_shares: self.invalid_shares.load(Ordering::Relaxed),
            uptime_secs: self.start_time.elapsed().as_secs(),
            current_wallet: identity.wallet.clone(),
            active_wallet_masked: Self::mask_wallet(&identity.wallet),
            active_target_type: identity.target_type,
            dev_fee_state: *self.dev_fee_state.read().await,
            job_age_secs: self.last_job_time.read().await.elapsed().as_secs(),
        }
    }

    pub async fn set_identity(&self, identity: ActiveMiningIdentity) {
        let mut id = self.current_identity.write().await;
        *id = identity;
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
                wallet: "1234567890abcdef".to_string(),
                worker: "worker1".to_string(),
                target_type: MiningTargetType::User,
            })
            .await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.candidates_found, 1);
        assert_eq!(stats.pool_accepted_shares, 1);
        assert_eq!(stats.pool_rejected_shares, 1);
        assert_eq!(stats.stale_shares, 1);
        assert_eq!(stats.hashrate, 1234.5);
        assert_eq!(stats.current_wallet, "1234567890abcdef");
        assert_eq!(stats.active_wallet_masked, "123456...cdef");
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
