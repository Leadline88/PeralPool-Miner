use process::ProcessManager;
use std::time::{Duration, Instant};
use tokio::signal;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct WatchdogConfig {
    pub restart_delay: Duration,
    pub max_restarts_per_window: usize,
    pub window_duration: Duration,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            restart_delay: Duration::from_secs(3),
            max_restarts_per_window: 5,
            window_duration: Duration::from_secs(60),
        }
    }
}

pub struct WatchdogState {
    pub uptime: Instant,
    pub last_crash_reason: Option<i32>,
    pub restart_timestamps: Vec<Instant>,
}

impl Default for WatchdogState {
    fn default() -> Self {
        Self {
            uptime: Instant::now(),
            last_crash_reason: None,
            restart_timestamps: Vec::new(),
        }
    }
}

pub struct Watchdog;

impl Watchdog {
    pub async fn run(binary_path: String, args: Vec<String>) {
        let config = WatchdogConfig::default();
        Self::run_with_config(config, binary_path, args).await;
    }

    pub async fn run_with_config(config: WatchdogConfig, binary_path: String, args: Vec<String>) {
        let mut state = WatchdogState::default();

        loop {
            info!("Watchdog: Starting miner...");
            let child_result = ProcessManager::spawn(&binary_path, &args).await;

            match child_result {
                Ok(mut child) => {
                    tokio::select! {
                        status = child.wait() => {
                            match status {
                                Ok(exit_status) => {
                                    warn!("Miner process exited with status: {}", exit_status);
                                    state.last_crash_reason = exit_status.code();
                                }
                                Err(e) => {
                                    error!("Failed to wait on miner process: {}", e);
                                    state.last_crash_reason = None;
                                }
                            }

                            info!("Watchdog: Uptime: {} seconds", state.uptime.elapsed().as_secs());
                            if let Some(code) = state.last_crash_reason {
                                info!("Watchdog: Last crash reason (exit code): {}", code);
                            }

                            let now = Instant::now();
                            state.restart_timestamps.retain(|&t| now.duration_since(t) < config.window_duration);

                            if state.restart_timestamps.len() >= config.max_restarts_per_window {
                                error!("Watchdog: Restart limit reached ({} restarts in the last {} seconds). Giving up.", config.max_restarts_per_window, config.window_duration.as_secs());
                                break;
                            }

                            state.restart_timestamps.push(now);
                            info!("Watchdog: Restarting miner in {} seconds...", config.restart_delay.as_secs());
                            sleep(config.restart_delay).await;
                        }
                        _ = signal::ctrl_c() => {
                            info!("Watchdog: Received Ctrl+C. Shutting down...");
                            if let Err(e) = child.kill().await {
                                error!("Failed to kill miner process: {}", e);
                            }
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Watchdog: Failed to start miner: {}", e);

                    let now = Instant::now();
                    state
                        .restart_timestamps
                        .retain(|&t| now.duration_since(t) < config.window_duration);
                    if state.restart_timestamps.len() >= config.max_restarts_per_window {
                        error!("Watchdog: Start limit reached. Giving up.");
                        break;
                    }
                    state.restart_timestamps.push(now);

                    info!(
                        "Watchdog: Retrying in {} seconds...",
                        config.restart_delay.as_secs()
                    );
                    sleep(config.restart_delay).await;
                }
            }
        }
        info!("Watchdog: Exited gracefully.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_restart_limit() {
        let config = WatchdogConfig {
            restart_delay: Duration::from_millis(10),
            max_restarts_per_window: 3,
            window_duration: Duration::from_secs(10),
        };
        let mut state = WatchdogState::default();

        let now = Instant::now();

        // Simulate 3 rapid crashes
        for _ in 0..3 {
            state
                .restart_timestamps
                .retain(|&t| now.duration_since(t) < config.window_duration);
            assert!(state.restart_timestamps.len() < config.max_restarts_per_window);
            state.restart_timestamps.push(Instant::now());
        }

        // The 4th crash should trigger the limit
        state
            .restart_timestamps
            .retain(|&t| now.duration_since(t) < config.window_duration);
        assert_eq!(
            state.restart_timestamps.len(),
            config.max_restarts_per_window
        );
    }

    #[tokio::test]
    async fn test_watchdog_with_dummy_process() {
        let config = WatchdogConfig {
            restart_delay: Duration::from_millis(10),
            max_restarts_per_window: 2,
            window_duration: Duration::from_secs(10),
        };

        // Use 'false' which always exits with status 1
        let binary_path = if cfg!(windows) {
            "cmd".to_string()
        } else {
            "false".to_string()
        };

        let args = if cfg!(windows) {
            vec!["/C".to_string(), "exit 1".to_string()]
        } else {
            vec![]
        };

        let start = Instant::now();
        // This should hit the restart limit and exit the loop
        Watchdog::run_with_config(config, binary_path, args).await;

        let elapsed = start.elapsed();
        // Since restart limit is 2 and delay is 10ms, it should take at least 20ms
        // It'll attempt to run once, crash, wait 10ms, attempt again, crash, wait 10ms, attempt again, crash, and then stop
        assert!(elapsed >= Duration::from_millis(20));
    }
}
