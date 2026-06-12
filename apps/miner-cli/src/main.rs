use clap::{Parser, ValueEnum};
use config::{Config, MiningBackend, MiningMode};
use std::path::PathBuf;
use std::process::exit;
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
    benchmark: bool,

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

impl From<Backend> for MiningBackend {
    fn from(backend: Backend) -> Self {
        match backend {
            Backend::Cpu => MiningBackend::Cpu,
            Backend::Cuda => MiningBackend::Cuda,
            Backend::Hip => MiningBackend::Hip,
            Backend::Opencl => MiningBackend::Opencl,
            Backend::Sycl => MiningBackend::Sycl,
            Backend::Metal => MiningBackend::Metal,
        }
    }
}

#[tokio::main]
async fn main() {
    logging::init();
    let args = Args::parse();

    if args.dev_fee_info {
        println!("Pearl Miner Developer Fee Information:");
        println!("Default fee: {}%", devfee::DEFAULT_DEV_FEE);
        println!("Fee wallet: {}", devfee::DEFAULT_DEV_WALLET);
        println!("The developer fee is used to support the ongoing development of Pearl Miner.");
        println!("It is transparently integrated into the mining process.");
        return;
    }

    info!("Starting Pearl Miner...");
    info!("Developer fee: {}%", devfee::DEFAULT_DEV_FEE);

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
    if args.benchmark {
        config.benchmark = true;
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

    info!("Mode: {:?}", config.mode);
    info!("Backend: {:?}", config.backend);
    info!("Algorithm: {}", config.algo);
    info!("Worker: {}", config.worker_name);

    match config.mode {
        MiningMode::Compatibility => {
            info!("Running in compatibility mode (external miner)");
            Watchdog::run(config.miner_binary_path, config.args).await;
        }
        MiningMode::NativeCpu => {
            info!("Running in native-cpu mode");
            if config.benchmark {
                info!("Running benchmark...");
                // Placeholder for benchmark
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                info!("Benchmark completed: 0.00 H/s (Not implemented)");
            } else if config.dry_run {
                info!("Dry run enabled, exiting.");
            } else {
                info!("Native CPU mining started (Placeholder)");
                // Placeholder for native mining loop
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    info!("Status: 0.00 H/s, Shares: 0, Fee: {}%", devfee::DEFAULT_DEV_FEE);
                }
            }
        }
        MiningMode::NativeGpu => {
            info!("Running in native-gpu mode");
            info!("Backend: {:?}", config.backend);
            error!("Native GPU mining is not yet implemented for {:?} backend.", config.backend);
            exit(1);
        }
    }
}
