use algo_pearl::PearlShareCandidate;
use backend_cpu::CpuBackend;
use chrono::Utc;
use clap::{Parser, ValueEnum};
use config::{Config, MiningBackend as ConfigBackend, MiningMode};
use mining::{MiningAlgorithm, MiningBackend};
use pearlpool::PearlPoolAdapter;
use scheduler::DevFeeScheduler;
use stats::StatsManager;
use std::path::PathBuf;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use stratum::client::StratumClient;
use stratum::miner_loop::MinerLoop;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use watchdog::Watchdog;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mining mode
    #[arg(long, value_enum)]
    mode: Option<Mode>,

    /// Mining backend
    #[arg(long, value_enum)]
    backend: Option<Backend>,

    /// Mining algorithm
    #[arg(long)]
    algo: Option<String>,

    /// Wallet address
    #[arg(long)]
    wallet: Option<String>,

    /// Worker name
    #[arg(long)]
    worker: Option<String>,

    /// Pool URL
    #[arg(long)]
    pool: Option<String>,

    /// Path to external miner binary (for compatibility mode)
    #[arg(long)]
    miner_binary: Option<String>,

    /// Display developer fee information
    #[arg(long)]
    dev_fee_info: bool,

    /// Run benchmark
    #[arg(long)]
    benchmark_native_cpu: bool,

    /// Number of threads for native-cpu mode
    #[arg(long)]
    threads: Option<usize>,

    /// Enable deterministic mode for testing
    #[arg(long)]
    deterministic: bool,

    /// Verify a Pearl fixture
    #[arg(long)]
    verify_pearl_fixture: Option<String>,

    /// Dry run (no actual mining)
    #[arg(long)]
    dry_run: bool,

    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Mode {
    Compatibility,
    NativeCpu,
    NativeGpu,
}

impl From<Mode> for MiningMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Compatibility => MiningMode::Compatibility,
            Mode::NativeCpu => MiningMode::NativeCpu,
            Mode::NativeGpu => MiningMode::NativeGpu,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Backend {
    Cpu,
    Cuda,
    Hip,
    Opencl,
    Sycl,
    Metal,
}

impl From<Backend> for ConfigBackend {
    fn from(backend: Backend) -> Self {
        match backend {
            Backend::Cpu => ConfigBackend::Cpu,
            Backend::Cuda => ConfigBackend::Cuda,
            Backend::Hip => ConfigBackend::Hip,
            Backend::Opencl => ConfigBackend::Opencl,
            Backend::Sycl => ConfigBackend::Sycl,
            Backend::Metal => ConfigBackend::Metal,
        }
    }
}

#[tokio::main]
async fn main() {
    logging::init();
    let args = Args::parse();

    if args.dev_fee_info {
        print_dev_fee_info();
        return;
    }

    if args.benchmark_native_cpu {
        run_benchmark().await;
        return;
    }

    if let Some(path) = args.verify_pearl_fixture {
        verify_fixture(&path).await;
        return;
    }

    let mut config = if args.config.exists() {
        match Config::load_from_file(&args.config) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load configuration from {:?}: {}", args.config, e);
                exit(1);
            }
        }
    } else {
        Config::default()
    };

    // Override config with CLI arguments if provided
    if let Some(wallet) = args.wallet {
        config.wallet = wallet;
    }
    if let Some(worker) = args.worker {
        config.worker_name = worker;
    }
    if let Some(pool) = args.pool {
        config.pool_url = pool;
    }
    if let Some(miner_binary) = args.miner_binary {
        config.miner_binary_path = miner_binary;
    }
    if let Some(mode) = args.mode {
        config.mode = mode.into();
    }
    if let Some(backend) = args.backend {
        config.backend = backend.into();
    }
    if let Some(algo) = args.algo {
        config.algo = algo;
    }
    if let Some(threads) = args.threads {
        config.threads = threads;
    }
    if args.deterministic {
        config.deterministic = true;
    }
    if args.dry_run {
        config.dry_run = true;
    }

    // Validate merged config
    if let Err(e) = config.validate() {
        error!("Configuration validation failed: {}", e);
        error!("Please provide missing values via config file or CLI arguments.");
        exit(1);
    }

    info!("Starting Pearl Miner...");
    info!("Mode: {:?}", config.mode);
    info!("Backend: {:?}", config.backend);
    info!("Algorithm: {}", config.algo);
    info!("Worker: {}", config.worker_name);
    info!("Developer fee: 1.0%");

    match config.mode {
        MiningMode::Compatibility => {
            info!("Running in compatibility mode (external miner)");
            Watchdog::run(config.miner_binary_path, config.args).await;
        }
        MiningMode::NativeCpu => {
            info!("Running in native-cpu mode");
            if config.dry_run {
                info!("Dry run enabled, exiting.");
                return;
            }

            let stats = Arc::new(StatsManager::new());
            let is_dev_mining = Arc::new(AtomicBool::new(false));
            let scheduler =
                DevFeeScheduler::new(config.wallet.clone(), stats.clone(), is_dev_mining.clone());
            let threads = if config.threads > 0 {
                config.threads
            } else {
                num_cpus::get()
            };

            let (share_tx, mut share_rx) = mpsc::channel::<PearlShareCandidate>(100);

            let backend = Arc::new(CpuBackend::new(
                threads,
                stats.clone(),
                is_dev_mining.clone(),
                config.deterministic,
                share_tx,
            ));

            let scheduler_handle = tokio::spawn(async move {
                scheduler.run().await;
            });

            backend.start().await.expect("Failed to start backend");

            // Stratum Client setup
            let cancel_token = CancellationToken::new();
            let adapter = Arc::new(PearlPoolAdapter);
            let client = StratumClient::new(&config.pool_url, adapter.clone());
            let miner_loop = Arc::new(MinerLoop::new(
                client.clone(),
                adapter.clone(),
                config.wallet.clone(),
                config.worker_name.clone(),
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

            // Status display loop
            let stats_clone = stats.clone();
            let is_dev_mining_clone = is_dev_mining.clone();
            let backend_status_clone = backend.clone();
            let start_time = Utc::now();
            tokio::spawn(async move {
                let mut total_hashes = 0.0;
                let mut status_count = 0;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    let runtime_stats = stats_clone.get_stats().await;
                    let uptime = Utc::now() - start_time;
                    let hashes = backend_status_clone.get_hashrate().await;
                    total_hashes += hashes;
                    status_count += 1;

                    let hashrate = hashes / 10.0;
                    let avg_hashrate = total_hashes / (status_count as f64 * 10.0);

                    let target_type = if is_dev_mining_clone.load(Ordering::Relaxed) {
                        "DEVELOPER"
                    } else {
                        "USER"
                    };

                    info!(
                        "Status: {:.2} H/s (avg {:.2} H/s) | A: {} R: {} S: {} | Uptime: {} | Job Age: {}s | Target: {}",
                        hashrate,
                        avg_hashrate,
                        runtime_stats.accepted_shares,
                        runtime_stats.rejected_shares,
                        runtime_stats.stale_shares,
                        format_duration(uptime),
                        runtime_stats.job_age_secs,
                        target_type
                    );
                }
            });

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
        MiningMode::NativeGpu => {
            info!("Running in native-gpu mode");
            info!("Backend: {:?}", config.backend);
            error!(
                "Native GPU mining is not yet implemented for {:?} backend.",
                config.backend
            );
            exit(1);
        }
    }
}

fn format_duration(dur: chrono::Duration) -> String {
    let secs = dur.num_seconds();
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

async fn run_benchmark() {
    info!("Benchmarking Native CPU...");
    let algo = algo_pearl::PearlAlgorithm;
    let dummy_job_json = r#"{"id":"bench","blob":"00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff","target":1000000}"#;

    // Benchmark Job Parsing
    let parse_start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = algo.parse_job(dummy_job_json).unwrap();
    }
    let parse_time_avg = parse_start.elapsed().as_secs_f64() / 1000.0;

    // Benchmark Work Package Creation
    let work_start = std::time::Instant::now();
    let job = algo.parse_job(dummy_job_json).unwrap();
    for _ in 0..1000 {
        let _ = algo.create_work(
            &job,
            mining::NonceRange {
                start: 0,
                end: 1000,
            },
        );
    }
    let work_time_avg = work_start.elapsed().as_secs_f64() / 1000.0;

    // Benchmark Share Verification
    let verify_start = std::time::Instant::now();
    let blob = vec![0u8; 32];
    for i in 0..10000 {
        let _ = algo_pearl::PearlVerifier::verify(&blob, i as u64, 1000000);
    }
    let verify_time_avg = verify_start.elapsed().as_secs_f64() / 10000.0;

    // Benchmark Throughput - Single-threaded
    info!("Running single-threaded throughput benchmark (5 seconds)...");
    let start = std::time::Instant::now();
    let mut count = 0;
    let target = 0x00000000FFFFFFFF;
    while start.elapsed().as_secs() < 5 {
        for _ in 0..10000 {
            algo_pearl::PearlVerifier::verify(&blob, count, target);
            count += 1;
        }
    }
    let elapsed_single = start.elapsed().as_secs_f64();
    let hashrate_single = count as f64 / elapsed_single;
    info!("Single-threaded result: {:.2} H/s", hashrate_single);

    // Benchmark Throughput - Multi-threaded
    let threads = num_cpus::get();
    info!(
        "Running multi-threaded ({} threads) throughput benchmark (5 seconds)...",
        threads
    );
    let start = std::time::Instant::now();
    let total_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut handles = Vec::new();

    for i in 0..threads {
        let total_count = total_count.clone();
        let blob = blob.clone();
        handles.push(std::thread::spawn(move || {
            let mut local_count = 0;
            let mut nonce = i as u64;
            let thread_start = std::time::Instant::now();
            while thread_start.elapsed().as_secs() < 5 {
                for _ in 0..10000 {
                    algo_pearl::PearlVerifier::verify(&blob, nonce, target);
                    nonce += threads as u64;
                    local_count += 1;
                }
            }
            total_count.fetch_add(local_count, std::sync::atomic::Ordering::Relaxed);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed_multi = start.elapsed().as_secs_f64();
    let hashrate_multi =
        total_count.load(std::sync::atomic::Ordering::Relaxed) as f64 / elapsed_multi;
    info!("Multi-threaded result: {:.2} H/s", hashrate_multi);

    let report = serde_json::json!({
        "backend": "native-cpu",
        "hashrate_single": hashrate_single,
        "hashrate_multi": hashrate_multi,
        "unit": "H/s",
        "threads": threads,
        "duration_secs": 5.0,
        "avg_job_parse_secs": parse_time_avg,
        "avg_work_package_creation_secs": work_time_avg,
        "avg_share_verify_secs": verify_time_avg,
        "timestamp": Utc::now().to_rfc3339()
    });

    println!("{}", serde_json::to_string_pretty(&report).unwrap());

    // Write report to file
    let report_path = "benchmark_report.json";
    if let Ok(content) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write(report_path, content);
        info!("Benchmark report saved to {}", report_path);
    }

    info!("Benchmark complete!");
}

async fn verify_fixture(path: &str) {
    info!("Verifying fixture: {}", path);
    let content = std::fs::read_to_string(path).expect("Failed to read fixture");
    let fixture: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse fixture");

    let blob = hex::decode(fixture["blob"].as_str().unwrap()).unwrap();
    let target = fixture["target"].as_u64().unwrap();

    if let Some(expected_shares) = fixture["expected_shares"].as_array() {
        for share in expected_shares {
            let nonce = share["nonce"].as_u64().unwrap();
            let is_valid = algo_pearl::PearlVerifier::verify(&blob, nonce, target);
            if is_valid {
                info!("Fixture Share [nonce: {}] - VALID", nonce);
            } else {
                error!("Fixture Share [nonce: {}] - INVALID", nonce);
                std::process::exit(1);
            }
        }
    }
    info!("Fixture verification successful!");
}

fn print_dev_fee_info() {
    println!("Pearl Miner Developer Fee Information:");
    println!("Default fee: 1.0%");
    println!("Fee wallet: 1DevFeeAddressExample");
    println!("The developer fee is used to support the ongoing development of Pearl Miner.");
    println!("It is transparently integrated into the mining process.");
}
