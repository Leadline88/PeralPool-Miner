use crate::output::start_status_loop;
use algo_pearl::PearlShareCandidate;
use backend_cpu::CpuBackend;
use config::Config;
use mining::MiningBackend;
use stats::StatsManager;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub async fn run(config: Config) {
    warn!("NOTICE: Running in native-cpu mode (SYNTHETIC / REFERENCE-ONLY)");
    warn!("This implementation is NOT performance competitive and NOT verified for Pearl mainnet.");
    info!("Native CPU mode is offline synthetic/reference-only. Live PearlPool mining is not verified.");
    info!(
        "Developer-fee policy is defined, but active collection is not implemented in native mode."
    );

    if config.dry_run {
        info!("Dry run enabled, exiting.");
        return;
    }

    let stats = Arc::new(StatsManager::new());
    let threads = if config.threads > 0 {
        config.threads
    } else {
        num_cpus::get()
    };

    // We still use a stub fee state for the backend but don't run the scheduler
    let fee_state = Arc::new(tokio::sync::RwLock::new(
        scheduler::DevFeeScheduler::new(
            config.wallet.clone(),
            config.worker_name.clone(),
            stats.clone(),
        )
        .get_state(),
    ));

    let (share_tx, mut share_rx) = mpsc::channel::<PearlShareCandidate>(100);

    let backend = Arc::new(CpuBackend::new(
        threads,
        stats.clone(),
        fee_state.clone(),
        config.deterministic,
        share_tx,
    ));

    backend.start().await.expect("Failed to start backend");

    // Offline synthetic job
    let dummy_job = serde_json::json!({
        "id": "offline-synthetic",
        "blob": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
        "target": 1000000
    });
    backend
        .set_job(&dummy_job.to_string())
        .await
        .expect("Failed to set synthetic job");

    // Drain shares to avoid blocking backend
    tokio::spawn(async move {
        while share_rx.recv().await.is_some() {
            // In offline mode, we just drop shares or could count them locally
        }
    });

    start_status_loop(
        stats,
        None,
        backend.clone(),
        config.mode,
        config.backend,
        false,
    );

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    info!("Received Ctrl+C, shutting down...");
    backend.stop().await.expect("Failed to stop backend");
}
