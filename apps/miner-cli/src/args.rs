use clap::{Parser, ValueEnum};
use config::{MiningBackend as ConfigBackend, MiningMode};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Mining mode
    #[arg(long, value_enum)]
    pub mode: Option<Mode>,

    /// Mining backend
    #[arg(long, value_enum)]
    pub backend: Option<Backend>,

    /// Mining algorithm
    #[arg(long)]
    pub algo: Option<String>,

    /// Wallet address
    #[arg(long)]
    pub wallet: Option<String>,

    /// Worker name
    #[arg(long)]
    pub worker: Option<String>,

    /// Pool URL
    #[arg(long)]
    pub pool: Option<String>,

    /// Path to external miner binary (for compatibility mode)
    #[arg(long)]
    pub miner_binary: Option<String>,

    /// Display developer fee information
    #[arg(long)]
    pub dev_fee_info: bool,

    /// Run benchmark
    #[arg(long)]
    pub benchmark_native_cpu: bool,

    /// Number of threads for native-cpu mode
    #[arg(long)]
    pub threads: Option<usize>,

    /// Enable deterministic mode for testing
    #[arg(long)]
    pub deterministic: bool,

    /// Verify a Pearl fixture
    #[arg(long)]
    pub verify_pearl_fixture: Option<String>,

    /// Dry run (no actual mining)
    #[arg(long)]
    pub dry_run: bool,

    /// Profile to use from configuration
    #[arg(long)]
    pub profile: Option<String>,

    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Validate configuration and exit
    #[arg(long)]
    pub validate_config: bool,

    /// Print command for compatibility mode and exit
    #[arg(long)]
    pub print_command: bool,

    /// Allow experimental live Stratum mining (unverified)
    #[arg(long)]
    pub allow_experimental_live_stratum: bool,

    /// Optional benchmark report output path
    #[arg(long)]
    pub benchmark_output: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
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
pub enum Backend {
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
