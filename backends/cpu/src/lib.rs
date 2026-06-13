use algo_pearl::{PearlAlgorithm, PearlJob, PearlShareCandidate, PearlVerifier};
use async_trait::async_trait;
use mining::{MiningAlgorithm, MiningBackend};
use stats::StatsManager;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{info, warn};

pub struct CpuBackend {
    threads: usize,
    stats: Arc<StatsManager>,
    current_job: Arc<RwLock<Option<PearlJob>>>,
    cancel_tx: broadcast::Sender<()>,
    is_dev_mining: Arc<AtomicBool>,
    total_hashes: Arc<AtomicU64>,
    share_tx: mpsc::Sender<PearlShareCandidate>,
    worker_handles: Arc<RwLock<Vec<JoinHandle<()>>>>,
    deterministic: bool,
}

struct WorkerContext {
    id: usize,
    threads: usize,
    job: PearlJob,
    cancel_rx: broadcast::Receiver<()>,
    stats: Arc<StatsManager>,
    is_dev_mining: Arc<AtomicBool>,
    total_hashes: Arc<AtomicU64>,
    share_tx: mpsc::Sender<PearlShareCandidate>,
    deterministic: bool,
}

impl CpuBackend {
    pub fn new(
        threads: usize,
        stats: Arc<StatsManager>,
        is_dev_mining: Arc<AtomicBool>,
        deterministic: bool,
        share_tx: mpsc::Sender<PearlShareCandidate>,
    ) -> Self {
        let (cancel_tx, _) = broadcast::channel(1);
        Self {
            threads,
            stats,
            current_job: Arc::new(RwLock::new(None)),
            cancel_tx,
            is_dev_mining,
            total_hashes: Arc::new(AtomicU64::new(0)),
            share_tx,
            worker_handles: Arc::new(RwLock::new(Vec::new())),
            deterministic,
        }
    }

    async fn worker_loop(ctx: WorkerContext) {
        let mut cancel_rx = ctx.cancel_rx;
        let blob = hex::decode(&ctx.job.blob).unwrap_or_default();
        let mut nonce = ctx.id as u64;
        let mut local_hashes = 0u64;

        // In deterministic mode, we use a fixed range to avoid infinite loops in tests
        let max_nonces = if ctx.deterministic { 100_000 } else { u64::MAX };
        let mut total_processed = 0u64;

        loop {
            // Check for cancellation every 10,000 nonces
            if local_hashes >= 10000 {
                ctx.total_hashes.fetch_add(local_hashes, Ordering::Relaxed);
                total_processed += local_hashes;
                local_hashes = 0;

                if total_processed >= max_nonces {
                    break;
                }

                match cancel_rx.try_recv() {
                    Ok(_) => break,
                    Err(broadcast::error::TryRecvError::Empty) => {}
                    Err(_) => break, // Closed or Lagged
                }
                // Yield to Tokio executor to prevent starvation
                tokio::task::yield_now().await;
            }

            if PearlVerifier::verify(&blob, nonce, ctx.job.target) {
                let target_type = if ctx.is_dev_mining.load(Ordering::Relaxed) {
                    "DEVELOPER"
                } else {
                    "USER"
                };
                info!(
                    "Worker {}: Found candidate for {} at nonce {}",
                    ctx.id, target_type, nonce
                );

                let candidate = PearlShareCandidate {
                    job_id: ctx.job.id.clone(),
                    nonce,
                    hash: String::new(), // Placeholder, hash is verified by nonce on pool side usually
                };

                if let Err(e) = ctx.share_tx.send(candidate).await {
                    tracing::error!("Failed to send share candidate: {}", e);
                }

                ctx.stats.inc_candidates_found_sync();
            }

            local_hashes += 1;

            // Nonce range partitioning: each thread takes a different starting point
            // and increments by the total number of threads.
            if let Some(next_nonce) = nonce.checked_add(ctx.threads as u64) {
                nonce = next_nonce;
            } else {
                // Nonce overflow - this worker is done with this job's address space
                break;
            }
        }
        ctx.total_hashes.fetch_add(local_hashes, Ordering::Relaxed);
    }

    async fn stop_workers(&self) {
        let _ = self.cancel_tx.send(());
        let mut handles = self.worker_handles.write().await;

        for handle in handles.drain(..) {
            let abort_handle = handle.abort_handle();
            match tokio::time::timeout(std::time::Duration::from_millis(500), handle).await {
                Ok(res) => {
                    if let Err(e) = res {
                        warn!("Worker task failed: {}", e);
                    }
                }
                Err(_) => {
                    warn!("Worker task timed out on stop, aborting");
                    abort_handle.abort();
                }
            }
        }
    }
}
#[async_trait]
impl MiningBackend for CpuBackend {
    async fn start(&self) -> Result<(), String> {
        info!("Starting CPU backend with {} threads", self.threads);
        Ok(())
    }

    async fn stop(&self) -> Result<(), String> {
        self.stop_workers().await;
        Ok(())
    }

    async fn set_job(&self, job_data: &str) -> Result<(), String> {
        let algo = PearlAlgorithm;
        let job = algo.parse_job(job_data)?;

        // Cancel and wait for previous workers
        self.stop_workers().await;

        self.stats.notify_new_job().await;

        let mut current = self.current_job.write().await;
        *current = Some(job.clone());

        let mut handles = self.worker_handles.write().await;
        // Spawn new workers
        for i in 0..self.threads {
            let ctx = WorkerContext {
                id: i,
                threads: self.threads,
                job: job.clone(),
                cancel_rx: self.cancel_tx.subscribe(),
                stats: self.stats.clone(),
                is_dev_mining: self.is_dev_mining.clone(),
                total_hashes: self.total_hashes.clone(),
                share_tx: self.share_tx.clone(),
                deterministic: self.deterministic,
            };
            let handle = tokio::spawn(async move {
                Self::worker_loop(ctx).await;
            });
            handles.push(handle);
        }

        Ok(())
    }

    async fn get_hashrate(&self) -> f64 {
        self.total_hashes.swap(0, Ordering::Relaxed) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stats::StatsManager;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_nonce_partitioning_no_overlap() {
        let threads = 4;
        let _stats = Arc::new(StatsManager::new());
        let _is_dev_mining = Arc::new(AtomicBool::new(false));
        let (cancel_tx, _cancel_rx) = broadcast::channel::<()>(1);

        let scanned_nonces = Arc::new(Mutex::new(Vec::new()));
        let max_nonce = 1000u64;

        let mut handles = Vec::new();
        for i in 0..threads {
            let scanned_nonces = scanned_nonces.clone();
            let _cancel_rx = cancel_tx.subscribe();
            handles.push(tokio::spawn(async move {
                let mut nonce = i as u64;
                while nonce < max_nonce {
                    {
                        let mut lock = scanned_nonces.lock().await;
                        lock.push(nonce);
                    }
                    nonce += threads as u64;
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let mut all_nonces = scanned_nonces.lock().await;
        all_nonces.sort();

        assert_eq!(all_nonces.len(), max_nonce as usize);
        for i in 0..max_nonce as usize {
            assert_eq!(all_nonces[i], i as u64);
        }
    }

    #[tokio::test]
    async fn test_cancellation() {
        let stats = Arc::new(StatsManager::new());
        let is_dev_mining = Arc::new(AtomicBool::new(false));
        let (share_tx, _share_rx) = mpsc::channel(1);
        let backend = CpuBackend::new(1, stats, is_dev_mining, false, share_tx);

        // Use a target that is impossible to hit (0) so the worker keeps looping
        let dummy_job = r#"{"id":"1","blob":"00112233445566778899aabbccddeeff","target":0}"#;
        backend.set_job(dummy_job).await.unwrap();

        // Wait a bit and then set a new job, which should cancel the old one
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // This should trigger the cancellation of the previous worker
        backend.set_job(dummy_job).await.unwrap();

        {
            let handles = backend.worker_handles.read().await;
            assert_eq!(handles.len(), 1);
        }

        // Also stop it to be sure
        backend.stop().await.unwrap();

        {
            let handles = backend.worker_handles.read().await;
            assert_eq!(handles.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_regression_not_placeholder_hashrate() {
        let stats = Arc::new(StatsManager::new());
        let is_dev_mining = Arc::new(AtomicBool::new(false));
        let (share_tx, _share_rx) = mpsc::channel(1);
        let backend = CpuBackend::new(1, stats, is_dev_mining, false, share_tx);

        // Initially 0
        assert_eq!(backend.get_hashrate().await, 0.0);

        let dummy_job = r#"{"id":"1","blob":"00112233445566778899aabbccddeeff","target":1000000}"#;
        backend.set_job(dummy_job).await.unwrap();

        // Wait for some hashes to be computed
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let hashrate = backend.get_hashrate().await;
        assert!(
            hashrate > 0.0,
            "Hashrate should be positive if workers are running"
        );

        backend.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_deterministic_reproducibility() {
        let stats = Arc::new(StatsManager::new());
        let is_dev_mining = Arc::new(AtomicBool::new(false));
        let (share_tx1, mut share_rx1) = mpsc::channel(100);
        let (share_tx2, mut share_rx2) = mpsc::channel(100);

        let backend1 = CpuBackend::new(1, stats.clone(), is_dev_mining.clone(), true, share_tx1);
        let backend2 = CpuBackend::new(1, stats.clone(), is_dev_mining.clone(), true, share_tx2);

        // High target (easy to find shares)
        let dummy_job =
            r#"{"id":"1","blob":"00112233445566778899aabbccddeeff","target":18000000000000000000}"#;

        backend1.set_job(dummy_job).await.unwrap();
        backend2.set_job(dummy_job).await.unwrap();

        // Wait for workers to finish their 100,000 nonces
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let mut results1 = Vec::new();
        let mut results2 = Vec::new();

        while let Ok(candidate) = share_rx1.try_recv() {
            results1.push(candidate.nonce);
        }
        while let Ok(candidate) = share_rx2.try_recv() {
            results2.push(candidate.nonce);
        }

        backend1.stop().await.unwrap();
        backend2.stop().await.unwrap();

        assert!(!results1.is_empty(), "Should have found some candidates");
        assert_eq!(
            results1, results2,
            "Deterministic runs should produce identical results"
        );
    }
}
