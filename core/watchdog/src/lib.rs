use process::ProcessManager;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct Watchdog;

impl Watchdog {
    pub async fn run(binary_path: String, args: Vec<String>) {
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
                                }
                                Err(e) => {
                                    error!("Failed to wait on miner process: {}", e);
                                }
                            }
                            info!("Watchdog: Restarting miner in 3 seconds...");
                            sleep(Duration::from_secs(3)).await;
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
                    info!("Watchdog: Retrying in 5 seconds...");
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
        info!("Watchdog: Exited gracefully.");
    }
}
