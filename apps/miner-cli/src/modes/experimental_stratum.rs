use crate::output::start_status_loop;
use algo_pearl::PearlShareCandidate;
use backend_cpu::CpuBackend;
use chrono::Utc;
use config::Config;
use mining::MiningBackend;
use pearlpool::PearlPoolAdapter;
use scheduler::DevFeeScheduler;
use stats::StatsManager;
use std::sync::Arc;
use stratum::client::StratumClient;
use stratum::miner_loop::MinerLoop;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub async fn run(config: Config) {
    warn!("NOTICE: Running in native-cpu mode (EXPERIMENTAL LIVE STRATUM)");
    warn!("This implementation is NOT performance competitive and NOT verified for Pearl mainnet.");
    warn!("Live PearlPool mining is UNVERIFIED.");

    if config.dry_run {
        info!("Dry run enabled. Not connecting to pool.");
        return;
    }

    let stats = Arc::new(StatsManager::new());
    let scheduler = Arc::new(DevFeeScheduler::new(
        config.wallet.clone(),
        config.worker_name.clone(),
        stats.clone(),
    ));
    let fee_state = Arc::new(tokio::sync::RwLock::new(scheduler.get_state()));
    let threads = if config.threads > 0 {
        config.threads
    } else {
        num_cpus::get()
    };

    let (share_tx, mut share_rx) = mpsc::channel::<PearlShareCandidate>(100);

    let backend = Arc::new(CpuBackend::new(
        threads,
        stats.clone(),
        fee_state.clone(),
        config.deterministic,
        share_tx,
    ));

    let scheduler_clone = scheduler.clone();
    let fee_state_updater = fee_state.clone();
    let scheduler_handle = tokio::spawn(async move {
        // Update local fee_state periodically from scheduler
        let scheduler_for_updater = scheduler_clone.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let mut state = fee_state_updater.write().await;
                *state = scheduler_for_updater.get_state();
            }
        });
        scheduler_clone.run().await;
    });

    backend.start().await.expect("Failed to start backend");

    let cancel_token = CancellationToken::new();

    // Stratum Client setup
    let adapter = Arc::new(PearlPoolAdapter::new(true));
    let client = StratumClient::new(&config.pool_url, adapter.clone());
    let miner_loop = Arc::new(MinerLoop::new(
        client.clone(),
        adapter.clone(),
        config.wallet.clone(),
        config.worker_name.clone(),
        stats.clone(),
    ));

    let client_clone = client.clone();
    let cancel_token_clone = cancel_token.clone();
    tokio::spawn(async move {
        client_clone.run(cancel_token_clone).await;
    });

    let mut job_rx = miner_loop.subscribe_jobs();
    let backend_clone = backend.clone();
    tokio::spawn(async move {
        while let Some(job_value) = job_rx.recv().await {
            let job_str = job_value.to_string();
            if let Err(e) = backend_clone.set_job(&job_str).await {
                error!("Failed to set job in backend: {}", e);
            }
        }
    });

    let (miner_share_tx, miner_share_rx) = mpsc::channel(100);
    let miner_loop_clone = miner_loop.clone();
    let cancel_token_miner = cancel_token.clone();
    tokio::spawn(async move {
        miner_loop_clone
            .run(miner_share_rx, cancel_token_miner)
            .await;
    });

    let worker_name = config.worker_name.clone();
    tokio::spawn(async move {
        while let Some(candidate) = share_rx.recv().await {
            let share = shares::ShareCandidate {
                worker: worker_name.clone(),
                job_id: candidate.job_id,
                nonce: candidate.nonce.to_string(),
                timestamp: Utc::now(),
                result: String::new(),
            };
            let _ = miner_share_tx.send(share).await;
        }
    });

    start_status_loop(
        stats,
        Some(scheduler.clone()),
        backend.clone(),
        config.mode,
        config.backend,
        true,
    );

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = scheduler_handle => {
            error!("Scheduler task finished unexpectedly");
        }
    }

    cancel_token.cancel();
    backend.stop().await.expect("Failed to stop backend");
}
