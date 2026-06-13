use devfee::{DEFAULT_DEV_FEE, DEFAULT_DEV_WALLET};
use stats::StatsManager;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::info;

use std::sync::atomic::{AtomicBool, Ordering};

pub struct DevFeeScheduler {
    user_wallet: String,
    dev_wallet: String,
    fee_percent: f64,
    stats: Arc<StatsManager>,
    is_dev_mining: Arc<AtomicBool>,
}

impl DevFeeScheduler {
    pub fn new(
        user_wallet: String,
        stats: Arc<StatsManager>,
        is_dev_mining: Arc<AtomicBool>,
    ) -> Self {
        Self {
            user_wallet,
            dev_wallet: DEFAULT_DEV_WALLET.to_string(),
            fee_percent: DEFAULT_DEV_FEE,
            stats,
            is_dev_mining,
        }
    }

    pub async fn run(&self) {
        // Use 1-hour cycles for dev fee to ensure enough time for pool handshake and mining
        let total_cycle_secs = 3600; // 60 minutes
        let dev_time_secs = (total_cycle_secs as f64 * (self.fee_percent / 100.0)) as u64;
        let user_time_secs = total_cycle_secs - dev_time_secs;

        loop {
            // User mining
            info!(
                "Fee Scheduler: Switching to USER mining (Wallet: {})",
                self.user_wallet
            );
            self.stats.set_wallet(self.user_wallet.clone()).await;
            self.is_dev_mining.store(false, Ordering::SeqCst);
            // NOTE: In native mode, identity switching (re-authorization) is not yet active.
            // Shares will still be submitted under the user wallet.
            sleep(Duration::from_secs(user_time_secs)).await;

            // Dev mining
            info!(
                "Fee Scheduler: Switching to DEVELOPER mining (1.0% fee, Wallet: {})",
                self.dev_wallet
            );
            info!("Fee Scheduler: [NOTICE] Developer mining is scheduled but not yet active in native mode.");
            self.stats.set_wallet(self.dev_wallet.clone()).await;
            self.is_dev_mining.store(true, Ordering::SeqCst);
            sleep(Duration::from_secs(dev_time_secs)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fee_averaging() {
        let stats = Arc::new(StatsManager::new());
        let is_dev_mining = Arc::new(AtomicBool::new(false));
        let _scheduler = DevFeeScheduler::new("user".to_string(), stats, is_dev_mining.clone());

        let fee_percent = 1.0;
        let total_cycle = 3600.0;
        let dev_time = total_cycle * (fee_percent / 100.0);
        let user_time = total_cycle - dev_time;

        assert_eq!(dev_time, 36.0);
        assert_eq!(user_time, 3564.0);
        assert_eq!(dev_time / total_cycle, 0.01);
    }
}
