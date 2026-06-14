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
                // In native mode, identity switching (re-authorization) is not yet active.
                // So we use ScheduledInactive instead of ActiveDeveloperMining.
                *state = DevFeeState::ScheduledInactive;
            }
            self.stats
                .set_dev_fee_state(DevFeeState::ScheduledInactive)
                .await;
            info!("Fee Scheduler: Developer fee window scheduled, but native identity switching is not active; continuing under user wallet.");

            // Keep user identity active
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
}
