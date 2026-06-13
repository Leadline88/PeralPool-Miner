use serde::Serialize;
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
            job_age_secs: 0,
        }
    }
}

pub struct StatsManager {
    stats: Arc<RwLock<RuntimeStats>>,
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
            stats: Arc::new(RwLock::new(RuntimeStats::default())),
            start_time: Instant::now(),
            last_job_time: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn update_hashrate(&self, hashrate: f64) {
        let mut stats = self.stats.write().await;
        stats.hashrate = hashrate;
    }

    pub async fn inc_candidates_found(&self) {
        let mut stats = self.stats.write().await;
        stats.candidates_found += 1;
    }

    pub async fn inc_shares_submitted(&self) {
        let mut stats = self.stats.write().await;
        stats.shares_submitted += 1;
    }

    pub async fn inc_pool_accepted(&self) {
        let mut stats = self.stats.write().await;
        stats.pool_accepted_shares += 1;
    }

    pub async fn inc_pool_rejected(&self) {
        let mut stats = self.stats.write().await;
        stats.pool_rejected_shares += 1;
    }

    pub async fn inc_stale(&self) {
        let mut stats = self.stats.write().await;
        stats.stale_shares += 1;
    }

    pub async fn inc_invalid(&self) {
        let mut stats = self.stats.write().await;
        stats.invalid_shares += 1;
    }

    pub fn inc_candidates_found_sync(&self) {
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut s = stats.write().await;
            s.candidates_found += 1;
        });
    }

    pub fn inc_pool_accepted_sync(&self) {
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut s = stats.write().await;
            s.pool_accepted_shares += 1;
        });
    }

    pub fn inc_pool_rejected_sync(&self) {
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut s = stats.write().await;
            s.pool_rejected_shares += 1;
        });
    }

    pub fn inc_stale_sync(&self) {
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut s = stats.write().await;
            s.stale_shares += 1;
        });
    }

    pub async fn get_stats(&self) -> RuntimeStats {
        let mut stats = self.stats.read().await.clone();
        stats.uptime_secs = self.start_time.elapsed().as_secs();
        stats.job_age_secs = self.last_job_time.read().await.elapsed().as_secs();
        stats
    }

    pub async fn set_wallet(&self, wallet: String) {
        let mut stats = self.stats.write().await;
        stats.current_wallet = wallet;
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
        manager.inc_candidates_found().await;
        manager.inc_pool_accepted().await;
        manager.inc_pool_rejected().await;
        manager.inc_stale().await;
        manager.update_hashrate(1234.5).await;
        manager.set_wallet("test_wallet".to_string()).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.candidates_found, 1);
        assert_eq!(stats.pool_accepted_shares, 1);
        assert_eq!(stats.pool_rejected_shares, 1);
        assert_eq!(stats.stale_shares, 1);
        assert_eq!(stats.hashrate, 1234.5);
        assert_eq!(stats.current_wallet, "test_wallet");
    }

    #[tokio::test]
    async fn test_job_age() {
        let manager = StatsManager::new();
        manager.notify_new_job().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let stats = manager.get_stats().await;
        assert!(stats.job_age_secs >= 1);
    }
}
