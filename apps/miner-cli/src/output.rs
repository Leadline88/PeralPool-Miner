use backend_cpu::CpuBackend;
use chrono::Utc;
use config::{MiningBackend as ConfigBackend, MiningMode};
use mining::MiningBackend;
use scheduler::DevFeeScheduler;
use stats::StatsManager;
use std::sync::Arc;
use tracing::info;

pub fn start_status_loop(
    stats: Arc<StatsManager>,
    scheduler: Option<Arc<DevFeeScheduler>>,
    backend: Arc<CpuBackend>,
    mode: MiningMode,
    config_backend: ConfigBackend,
    is_live_experimental: bool,
) {
    let start_time = Utc::now();
    let is_live_str = if is_live_experimental {
        "live (experimental/unverified)"
    } else {
        "synthetic/reference-only"
    };

    tokio::spawn(async move {
        let mut total_hashes = 0.0;
        let mut status_count = 0;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let runtime_stats = stats.get_stats().await;
            let uptime = Utc::now() - start_time;
            let hashes = backend.get_hashrate().await;
            total_hashes += hashes;
            status_count += 1;

            let hashrate = hashes / 10.0;
            let avg_hashrate = total_hashes / (status_count as f64 * 10.0);

            let dev_fee_state = if let Some(s) = &scheduler {
                s.get_state().to_string()
            } else {
                "Disabled/Inactive (native mode)".to_string()
            };

            info!(
                "Status: Mode: {:?} | Backend: {:?} | Type: {} | ActiveTarget: {} | ActiveWallet: {} | DevFeeState: {}",
                mode, config_backend, is_live_str, runtime_stats.active_target_type, runtime_stats.active_wallet_masked, dev_fee_state
            );
            info!(
                "Status: Uptime: {} | Job Age: {}s | C: {} | S: {} | A: {} | R: {} | Stale: {} | Invalid: {} | U: {} | {:.2} H/s (avg {:.2} H/s)",
                format_duration(uptime),
                runtime_stats.job_age_secs,
                runtime_stats.candidates_found,
                runtime_stats.shares_submitted,
                runtime_stats.pool_accepted_shares,
                runtime_stats.pool_rejected_shares,
                runtime_stats.stale_shares,
                runtime_stats.invalid_shares,
                runtime_stats.unsupported_submit,
                hashrate,
                avg_hashrate
            );
            if scheduler.is_none() {
                info!("Status: Active developer-fee collection is not implemented in native mode.");
            }
        }
    });
}

pub fn format_duration(dur: chrono::Duration) -> String {
    let secs = dur.num_seconds();
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}
