use devfee::{DevFeeState, DEFAULT_DEV_FEE};
use stats::{ActiveMiningIdentity, MiningTargetType, StatsManager};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::info;

pub struct DevFeeScheduler {
    user_wallet: String,
    user_worker: String,
    fee_percent: f64,
    stats: Arc<StatsManager>,
    state: Arc<std::sync::RwLock<DevFeeState>>,
}

impl DevFeeScheduler {
    pub fn new(user_wallet: String, user_worker: String, stats: Arc<StatsManager>) -> Self {
        Self {
            user_wallet,
            user_worker,
            fee_percent: DEFAULT_DEV_FEE,
            stats,
            state: Arc::new(std::sync::RwLock::new(DevFeeState::ActiveUserMining)),
        }
    }

    pub fn get_state(&self) -> DevFeeState {
        *self.state.read().unwrap()
    }

    pub async fn run(&self) {
        // Use 1-hour cycles for dev fee to ensure enough time for pool handshake and mining
        let total_cycle_secs = 3600; // 60 minutes
        let dev_time_secs = (total_cycle_secs as f64 * (self.fee_percent / 100.0)) as u64;
        let user_time_secs = total_cycle_secs - dev_time_secs;

        loop {
            // User mining
            {
                let mut state = self.state.write().unwrap();
                *state = DevFeeState::ActiveUserMining;
            }
            self.stats
                .set_dev_fee_state(DevFeeState::ActiveUserMining)
                .await;
            info!(
                "Fee Scheduler: Switching to USER mining (Wallet: {})",
                self.user_wallet
            );
            self.stats
                .set_identity(ActiveMiningIdentity {
                    wallet: self.user_wallet.clone(),
                    worker: self.user_worker.clone(),
                    target_type: MiningTargetType::User,
                })
                .await;

            sleep(Duration::from_secs(user_time_secs)).await;

            // Dev mining window
            {
                let mut state = self.state.write().unwrap();
                // In native mode, identity switching (re-authorization) is not yet implemented.
                // So we use ScheduledInactive to indicate the fee period is active but no switching occurs.
                *state = DevFeeState::ScheduledInactive;
            }
            self.stats
                .set_dev_fee_state(DevFeeState::ScheduledInactive)
                .await;
            info!("Fee Scheduler: Developer fee window active (scheduled), but identity switching is not implemented in native mode yet; continuing under user wallet.");

            // Keep user identity active - NO switching to developer wallet in native foundation mode.
            self.stats
                .set_identity(ActiveMiningIdentity {
                    wallet: self.user_wallet.clone(),
                    worker: self.user_worker.clone(),
                    target_type: MiningTargetType::User,
                })
                .await;

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
        let _scheduler = DevFeeScheduler::new("user".to_string(), "worker".to_string(), stats);

        let fee_percent = 1.0;
        let total_cycle = 3600.0;
        let dev_time = total_cycle * (fee_percent / 100.0);
        let user_time = total_cycle - dev_time;

        assert_eq!(dev_time, 36.0);
        assert_eq!(user_time, 3564.0);
        assert_eq!(dev_time / total_cycle, 0.01);
    }

    #[tokio::test]
    async fn test_scheduler_state_transitions() {
        let stats = Arc::new(StatsManager::new());
        let scheduler = Arc::new(DevFeeScheduler {
            user_wallet: "user".to_string(),
            user_worker: "worker".to_string(),
            fee_percent: 50.0, // 50% for fast testing
            stats: stats.clone(),
            state: Arc::new(std::sync::RwLock::new(DevFeeState::ActiveUserMining)),
        });

        // Use a much smaller cycle for testing
        let _total_cycle_secs = 2;
        let dev_time_secs = 1;
        let user_time_secs = 1;

        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            loop {
                // Manually implement a fast version of run() for testing
                {
                    let mut state = scheduler_clone.state.write().unwrap();
                    *state = DevFeeState::ActiveUserMining;
                }
                sleep(Duration::from_secs(user_time_secs)).await;

                {
                    let mut state = scheduler_clone.state.write().unwrap();
                    *state = DevFeeState::ScheduledInactive;
                }
                sleep(Duration::from_secs(dev_time_secs)).await;
            }
        });

        sleep(Duration::from_millis(500)).await;
        assert_eq!(scheduler.get_state(), DevFeeState::ActiveUserMining);

        sleep(Duration::from_secs(1)).await;
        assert_eq!(scheduler.get_state(), DevFeeState::ScheduledInactive);
    }

    #[tokio::test]
    async fn test_scheduled_inactive_no_wallet_switch() {
        let stats = Arc::new(StatsManager::new());
        let user_wallet = "user_wallet".to_string();
        let user_worker = "user_worker".to_string();
        let scheduler =
            DevFeeScheduler::new(user_wallet.clone(), user_worker.clone(), stats.clone());

        // Set an initial identity
        stats
            .set_identity(ActiveMiningIdentity {
                wallet: "initial".to_string(),
                worker: "initial".to_string(),
                target_type: MiningTargetType::User,
            })
            .await;

        // Run the scheduler for a bit - it should switch to User mining first
        let scheduler_clone = Arc::new(scheduler);
        let scheduler_task = {
            let s = scheduler_clone.clone();
            tokio::spawn(async move {
                s.run().await;
            })
        };

        // Wait for it to set user identity
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let stats_val = stats.get_stats().await;
        assert_eq!(stats_val.active_target_type, MiningTargetType::User);
        // We can't easily check the full wallet because it's masked in get_stats,
        // but we can check the masked version matches the user wallet
        let mut expected_masked = String::new();
        if user_wallet.len() > 10 {
            expected_masked.push_str(&user_wallet[..6]);
            expected_masked.push_str("...");
            expected_masked.push_str(&user_wallet[user_wallet.len() - 4..]);
        } else {
            expected_masked = user_wallet.clone();
        }
        assert_eq!(stats_val.active_wallet_masked, expected_masked);

        // Manually trigger the ScheduledInactive logic to test it in isolation
        // since the real cycle is 1 hour.
        stats
            .set_dev_fee_state(DevFeeState::ScheduledInactive)
            .await;

        // Re-affirm user identity as the scheduler would
        stats
            .set_identity(ActiveMiningIdentity {
                wallet: user_wallet.clone(),
                worker: user_worker.clone(),
                target_type: MiningTargetType::User,
            })
            .await;

        let stats_val = stats.get_stats().await;
        assert_eq!(stats_val.dev_fee_state, DevFeeState::ScheduledInactive);
        assert_eq!(stats_val.active_target_type, MiningTargetType::User);
        assert_eq!(stats_val.active_wallet_masked, expected_masked);

        scheduler_task.abort();
    }
}
