use crate::adapter::{PoolAdapter, PoolAdapterError};
use crate::client::{StratumClient, StratumEvent};
use chrono::Utc;
use shares::{ShareCandidate, ShareResult, ShareStatus, ShareTracker};
use stats::StatsManager;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub struct MinerLoop {
    client: Arc<StratumClient>,
    adapter: Arc<dyn PoolAdapter>,
    wallet: String,
    worker: String,
    stats: Arc<StatsManager>,
    share_tracker: Arc<tokio::sync::Mutex<ShareTracker>>,
}

impl MinerLoop {
    pub fn new(
        client: Arc<StratumClient>,
        adapter: Arc<dyn PoolAdapter>,
        wallet: String,
        worker: String,
        stats: Arc<StatsManager>,
    ) -> Self {
        Self {
            client,
            adapter,
            wallet,
            worker,
            stats,
            share_tracker: Arc::new(tokio::sync::Mutex::new(ShareTracker::new())),
        }
    }

    pub fn subscribe_jobs(&self) -> mpsc::Receiver<serde_json::Value> {
        let mut job_rx = self.client.subscribe();
        let (job_tx, job_rx_out) = mpsc::channel(100);

        tokio::spawn(async move {
            while let Ok(event) = job_rx.recv().await {
                if let StratumEvent::Job(job) = event {
                    let _ = job_tx.send(job).await;
                }
            }
        });

        job_rx_out
    }

    pub async fn run(
        &self,
        mut share_rx: mpsc::Receiver<ShareCandidate>,
        cancel_token: CancellationToken,
    ) {
        let mut event_rx = self.client.subscribe();

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("Miner loop shutting down...");
                    break;
                }
                event = event_rx.recv() => {
                    match event {
                        Ok(StratumEvent::Connected) => {
                            info!("Miner loop: Connected, starting handshake...");

                            // 1. Subscribe
                            let sub_req = self.adapter.build_subscribe_request();
                            match self.client.send_request(sub_req).await {
                                Ok(res) => {
                                    if let Err(e) = self.adapter.parse_subscribe_response(&res) {
                                        error!("Subscribe failed: {}", e);
                                        continue;
                                    }
                                    info!("Subscribe successful");
                                }
                                Err(e) => {
                                    error!("Failed to send subscribe request: {}", e);
                                    continue;
                                }
                            }

                            // 2. Authorize (Login)
                            let login_req = self.adapter.build_login_request(&self.wallet, &self.worker);
                            match self.client.send_request(login_req).await {
                                Ok(res) => {
                                    match self.adapter.parse_login_response(&res) {
                                        Ok(true) => info!("Login successful"),
                                        Ok(false) => error!("Login failed (rejected by pool)"),
                                        Err(e) => error!("Failed to parse login response: {}", e),
                                    }
                                }
                                Err(e) => error!("Failed to send login request: {}", e),
                            }
                        }
                        Ok(StratumEvent::Disconnected) => {
                            warn!("Miner loop: Disconnected");
                        }
                        Ok(StratumEvent::Difficulty(diff)) => {
                            info!("Miner loop: Difficulty updated to {}", diff);
                            let mut tracker = self.share_tracker.lock().await;
                            tracker.set_difficulty(diff);
                        }
                        Ok(StratumEvent::Job(job)) => {
                            info!("Miner loop: New job received: {:?}", job);
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!("Miner loop: Missed {} events", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            error!("Miner loop: Event channel closed");
                            break;
                        }
                    }
                }
                Some(share) = share_rx.recv() => {
                    info!("Miner loop: Preparing to submit share for job {}", share.job_id);

                    let current_diff = {
                        let tracker = self.share_tracker.lock().await;
                        tracker.current_difficulty
                    };

                    let submit_req = match self.adapter.build_share_submit(&share) {
                        Ok(req) => req,
                        Err(e) => {
                            match e {
                                PoolAdapterError::UnsupportedRealPearlShareSubmitFormat => {
                                    warn!("Miner loop: {}", e);
                                    self.stats.inc_unsupported_submit();
                                }
                                _ => {
                                    error!("Miner loop: Error building share submit: {}", e);
                                    self.stats.inc_invalid();
                                }
                            }

                            let mut tracker = self.share_tracker.lock().await;
                            tracker.record_result(&ShareResult {
                                candidate: share,
                                status: ShareStatus::Invalid,
                                difficulty: current_diff,
                                latency_ms: 0,
                                error_message: Some(e.to_string()),
                            });
                            continue; // Skip actual submission
                        }
                    };

                    self.stats.inc_shares_submitted();
                    {
                        let mut tracker = self.share_tracker.lock().await;
                        tracker.record_submission();
                    }

                    let start = Utc::now();
                    let result = match self.client.send_request(submit_req).await {
                        Ok(res) => {
                            let latency = Utc::now().signed_duration_since(start).num_milliseconds() as u64;
                            match self.adapter.parse_share_response(&res) {
                                Ok(true) => {
                                    info!("Share accepted ({}ms)", latency);
                                    self.stats.inc_pool_accepted();
                                    ShareResult {
                                        candidate: share,
                                        status: ShareStatus::Accepted,
                                        difficulty: current_diff,
                                        latency_ms: latency,
                                        error_message: None,
                                    }
                                }
                                Ok(false) => {
                                    warn!("Share rejected ({}ms)", latency);
                                    self.stats.inc_pool_rejected();
                                    ShareResult {
                                        candidate: share,
                                        status: ShareStatus::Rejected,
                                        difficulty: current_diff,
                                        latency_ms: latency,
                                        error_message: Some("Rejected by pool".to_string()),
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to parse share response: {}", e);
                                    self.stats.inc_invalid();
                                    ShareResult {
                                        candidate: share,
                                        status: ShareStatus::Invalid,
                                        difficulty: current_diff,
                                        latency_ms: latency,
                                        error_message: Some(e),
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to submit share: {}", e);
                            self.stats.inc_stale();
                            ShareResult {
                                candidate: share,
                                status: ShareStatus::Stale, // Or connection error
                                difficulty: current_diff,
                                latency_ms: 0,
                                error_message: Some(e.to_string()),
                            }
                        }
                    };

                    let mut tracker = self.share_tracker.lock().await;
                    tracker.record_result(&result);
                }
            }
        }
    }

    pub fn get_tracker(&self) -> Arc<tokio::sync::Mutex<ShareTracker>> {
        self.share_tracker.clone()
    }
}
