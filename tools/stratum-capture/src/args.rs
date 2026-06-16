use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Standalone Stratum JSON-RPC proxy and capture tool", long_about = None)]
pub struct Args {
    /// Local address to listen on (e.g., 127.0.0.1:3333)
    #[arg(long, default_value = "127.0.0.1:3333")]
    pub listen: String,

    /// Upstream pool address (e.g., pearlpool.cloud:5566)
    #[arg(long)]
    pub pool: String,

    /// Output JSONL file path
    #[arg(long)]
    pub output: String,

    /// Wallet address to redact
    #[arg(long)]
    pub redact_wallet: Option<String>,

    /// Worker name to redact
    #[arg(long)]
    pub redact_worker: Option<String>,

    /// Password to redact
    #[arg(long)]
    pub redact_password: Option<String>,

    /// Optional label for the capture session
    #[arg(long)]
    pub session_label: Option<String>,

    /// Maximum number of lines to capture before stopping
    #[arg(long)]
    pub max_lines: Option<usize>,

    /// Maximum number of seconds to capture before stopping
    #[arg(long)]
    pub max_seconds: Option<u64>,

    /// Print pretty-formatted lines to stdout
    #[arg(long, default_value_t = false)]
    pub pretty_log: bool,

    /// Run without connecting or listening (validates args)
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

impl Args {
    pub fn validate(&self) {
        if self.listen.starts_with("0.0.0.0") || self.listen.starts_with("[::]") {
            eprintln!("WARNING: Listening on all interfaces ({}) is not recommended for a capture tool. It is safer to use 127.0.0.1.", self.listen);
        }
    }
}
